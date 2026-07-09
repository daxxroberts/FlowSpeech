# FlowSpeech — Embedded Cleanup LLM Implementation Plan

Goal: single FlowSpeech.exe — no helper process, no localhost server. The cleanup LLM
(Llama 3.1 8B Instruct, Wispr's base family) runs in-process, exactly like Parakeet does.

Decision register:
- Default cleanup model: **Llama-3.1-8B-Instruct Q4_K_M** (Daxx, 2026-07-06 — "use what
  Wispr uses": same base family; their fine-tune is proprietary and not downloadable).
- Qwen3-4B kept on disk as fast fallback if Llama fails the T1–T6 battery.
- Long game: LoRA fine-tune of the same base on Daxx's regression pairs (his data = the
  same recipe Wispr used). Out of scope for the embed; noted for later.

## Sequencing (why not embed first)

Phase A (now, helper process): prove model+prompt quality via llama-server — prompt
iteration costs seconds. Exit: Daxx prefers/ties FlowSpeech vs Wispr on 10+ regression
cases with cleanup ON.
Phase B (embed): freeze the proven config, build it into the exe. Prompt changes after
embed are still just settings edits — only the ENGINE moves in-process.

## Phase B — technical plan

### B1. Dependency: llama.cpp as a Rust dependency
- Crate: `llama-cpp-2` (maintained bindings over llama.cpp; supports Vulkan build flags)
  — evaluate vs direct FFI on a spike branch first (1 session).
- Build with GGML Vulkan backend, same generator rules as transcribe-cpp
  (Ninja + vcvars, CMAKE_GENERATOR=Ninja — see CLAUDE.md build notes).
- Risk: two ggml copies in one process (transcribe-cpp ships one, llama.cpp brings
  another). Check symbol isolation; llama-cpp-2 links statically by default which
  avoids DLL name collisions with the transcribe-cpp DLLs.

### B2. New manager: `managers/cleanup.rs`
- Mirror the ModelManager/TranscriptionManager pattern (AGENTS.md architecture):
  - lazy-load GGUF on first cleanup call (or eager at startup, setting-controlled)
  - unload on idle timeout (reuse `model_unload_timeout` pattern)
  - single-flight: one cleanup at a time, queue or drop-newest
- VRAM budget: Parakeet ~1 GB (it runs via ONNX on CPU by default — zero VRAM conflict
  in Daxx's config) + Llama 8B Q4 ~5.5 GB → fits 5070 Ti alongside everything.

### B3. Wire into the post-process path
- `managers/transcription.rs` post-process step: add provider variant `embedded`
  alongside the existing OpenAI-compatible HTTP providers.
- Settings: `post_process_provider_id: "embedded"`, model path + context size fields.
- Keep ALL existing HTTP providers (Ollama/BYO-key) — embedded is the default, not
  the only path. Offline/no-model fallback: rule-based filler stripper (already in
  the FlowSpeech spec) so dictation never hard-fails.

### B4. Model file distribution
- GGUF lives in app data models dir next to Parakeet (NOT bundled in installer —
  5 GB). Settings UI offers download like Whisper models do (reuse ModelManager
  download/verify flow, hf.co URL + sha256).

### B5. UI
- Settings → Cleanup: on/off, model picker (embedded models + "custom endpoint"),
  prompt editor (prefilled with CLEANUP_PROMPT.md v-current), short-utterance skip
  toggle (<~6 words skips LLM — locked decision).

### B6. Ship gates for the embed
- Same output as Phase A server for T1–T6 (bit-identical not required; rule-1
  violations = fail).
- Latency: cleanup adds ≤400 ms p50 on 10–30 word utterances (total release-to-text
  still <1 s median).
- Memory: no VRAM OOM with Parakeet + Llama loaded; graceful degradation if VRAM low.
- Task Manager shows ONE FlowSpeech process. Zero listening ports.

### Estimated effort (Claude does the work; Daxx approves + spot-tests)
- B1 spike: 1 session · B2+B3: 1–2 sessions · B4+B5: 1 session · gates+polish: 1 session
- Daxx's hands-on time: ~15 min total (UAC-free; test dictations after each build)
