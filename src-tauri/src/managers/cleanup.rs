//! Embedded cleanup LLM (FlowSpeech).
//!
//! Runs the cleanup model (Qwen3-4B GGUF) fully in-process via llama-cpp-2, so
//! FlowSpeech needs no helper server and nothing to download for post-processing.
//! This is the `embedded` post-process provider path in actions.rs.
//!
//! Lifetime/threading model: the backend + loaded model live in a process-global
//! behind a Mutex. All llama.cpp work happens synchronously inside that lock, and
//! callers invoke `run()` from `spawn_blocking` so inference never blocks the async
//! runtime. The per-request context is created and dropped inside `run()`, so there
//! is no self-referential struct to fight the borrow checker.

use anyhow::{anyhow, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use log::{debug, info};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Max tokens the cleanup pass may emit. Cleanup output is ~the same length as the
/// input transcript, so this is a generous safety ceiling, not a normal stop.
const MAX_NEW_TOKENS: i32 = 1024;
/// Context window for the cleanup model. Comfortably covers a long dictation plus
/// the system prompt.
const N_CTX: u32 = 4096;

struct CleanupEngine {
    backend: LlamaBackend,
    model: Option<LlamaModel>,
    /// Offload all layers to GPU when a backend is present.
    n_gpu_layers: u32,
}

static ENGINE: OnceLock<Mutex<CleanupEngine>> = OnceLock::new();
/// The model path is stored SEPARATELY from the engine so that startup can record
/// it without touching llama.cpp at all. `LlamaBackend::init()` is heavy (it scans
/// Vulkan devices) and MUST NOT run on the app-setup path — doing so blocked the
/// main window + shortcut + Enigo initialization. All llama.cpp init is deferred to
/// the first `run()`, which executes on a spawn_blocking worker thread.
static MODEL_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Lazily initialize the llama backend + model container. Only ever called from
/// `run()` on a worker thread — never at startup.
fn engine() -> &'static Mutex<CleanupEngine> {
    ENGINE.get_or_init(|| {
        let backend = LlamaBackend::init().expect("failed to init llama backend");
        Mutex::new(CleanupEngine {
            backend,
            model: None,
            n_gpu_layers: 999,
        })
    })
}

/// Record the on-disk GGUF path. Cheap and startup-safe: stores a string only, does
/// NOT initialize llama.cpp or load the model (both happen lazily on first cleanup).
pub fn set_model_path(path: PathBuf) {
    *MODEL_PATH.lock().unwrap() = Some(path);
}

/// True once a model file path is known AND the file exists on disk. Startup-safe.
pub fn is_available() -> bool {
    MODEL_PATH
        .lock()
        .unwrap()
        .as_ref()
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Build a Qwen3 (ChatML) prompt. We format manually rather than rely on the GGUF's
/// embedded template so behavior is identical to the benchmarked llama-server runs.
fn build_chatml(system: &str, user: &str) -> String {
    format!(
        "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n"
    )
}

/// Run one cleanup pass. Blocking — call from `spawn_blocking`. Returns the cleaned
/// text, or an error the caller can fall back from (raw transcript is kept on error).
pub fn run(system_prompt: &str, user_content: &str) -> Result<String> {
    // Resolve the path before touching llama.cpp so a missing model never triggers
    // backend init.
    let path = MODEL_PATH
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| anyhow!("cleanup model path not set"))?;
    if !path.exists() {
        return Err(anyhow!("cleanup model file missing: {}", path.display()));
    }

    // First lock here lazily initializes the backend (worker thread only).
    let mut eng = engine().lock().unwrap();

    // Lazy-load the model on first use.
    if eng.model.is_none() {
        info!("Loading embedded cleanup model: {}", path.display());
        let params = LlamaModelParams::default().with_n_gpu_layers(eng.n_gpu_layers);
        let load_start = std::time::Instant::now();
        let model = LlamaModel::load_from_file(&eng.backend, &path, &params)
            .map_err(|e| anyhow!("failed to load cleanup model: {e}"))?;
        info!("Embedded cleanup model loaded in {:?}", load_start.elapsed());
        eng.model = Some(model);
    }

    // Split the borrow so we can hold &backend and &model at once.
    let CleanupEngine {
        backend, model, ..
    } = &mut *eng;
    let model = model.as_ref().unwrap();

    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(N_CTX).unwrap()));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| anyhow!("failed to create cleanup context: {e}"))?;

    let prompt = build_chatml(system_prompt, user_content);
    let tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .map_err(|e| anyhow!("tokenize failed: {e}"))?;

    let n_ctx = ctx.n_ctx() as i32;
    if tokens.len() as i32 + MAX_NEW_TOKENS > n_ctx {
        return Err(anyhow!(
            "prompt too long for cleanup context ({} tokens)",
            tokens.len()
        ));
    }

    // Batch must hold the entire prompt at once; our system prompt alone is ~600
    // tokens, so size it to the context window rather than a fixed 512.
    let mut batch = LlamaBatch::new(N_CTX as usize, 1);
    let last = tokens.len() as i32 - 1;
    for (i, tok) in tokens.iter().enumerate() {
        batch
            .add(*tok, i as i32, &[0], i as i32 == last)
            .map_err(|e| anyhow!("batch add failed: {e}"))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| anyhow!("prompt decode failed: {e}"))?;

    // Greedy sampling = deterministic, matching the temp-0 benchmark runs.
    let mut sampler = LlamaSampler::greedy();

    let mut n_cur = batch.n_tokens();
    let mut out = String::new();
    let gen_start = std::time::Instant::now();

    while n_cur - batch.n_tokens() < MAX_NEW_TOKENS {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if model.is_eog_token(token) {
            break;
        }
        let piece = model
            .token_to_str(token, Special::Tokenize)
            .unwrap_or_default();
        out.push_str(&piece);

        // Qwen ChatML end marker — stop and trim if the model emits it as text.
        if let Some(idx) = out.find("<|im_end|>") {
            out.truncate(idx);
            break;
        }

        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| anyhow!("gen batch add failed: {e}"))?;
        n_cur += 1;
        ctx.decode(&mut batch)
            .map_err(|e| anyhow!("gen decode failed: {e}"))?;
    }

    debug!(
        "Embedded cleanup generated {} chars in {:?}",
        out.len(),
        gen_start.elapsed()
    );

    Ok(out.trim().to_string())
}
