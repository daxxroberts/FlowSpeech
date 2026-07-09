Read @AGENTS.md (upstream Handy dev guide — build commands, architecture, code style).

# FlowSpeech — Project Direction (read before any work)

## Mission
Recreate Wispr Flow for Daxx's personal use so he can cancel the $15/mo subscription.
Fork of Handy (MIT). The shell exists — the product is **processing quality + speed**.

Full plan + Blueprint Specification: `C:\Users\daxxr\.claude\plans\i-want-to-recreate-greedy-valley.md`

## Ship gate (do not soften these)
- **Latency:** median release-to-text < 1s, p95 < 1.5s, measured per-stage (asr_ms / llm_ms / inject_ms).
- **Quality:** regression suite (Daxx's real utterances vs Wispr's outputs, captured while still subscribed) — Daxx prefers ours or can't tell. Golden rule (refined by Daxx 2026-07-06): **polish, don't rewrite.** Light correction for clarity is welcome — normalize casual register ("gonna"→"going to"), fix grammar slips, smooth punctuation. The line: never reorder, rephrase, or substitute meaning-bearing words; whole-sentence rewrites are the failure mode. (Wispr itself lightly formalizes — that level is the target.)
- Wispr is not cancelled until both gates pass. SAGE (BOT team) holds the gate.

## Locked decisions (don't re-litigate; full register in the plan file)
- Name: **FlowSpeech** (Daxx, 2026-07-06).
- Hotkey: **Ctrl+Win** hold-to-talk (suppress Start-menu on Win keyup); fully rebindable.
- ASR: local only — Parakeet V3 default, Whisper Large V3 Turbo as accuracy alternative (both in Handy). No cloud ASR. Canary-Qwen parked (NeMo runtime unsupported).
- Cleanup LLM: TWO setup paths — (A) Local via Ollama/OpenAI-compatible localhost (model = user dropdown; 4B/8B/14B candidates on Daxx's RTX 5070 Ti 12GB), (B) bring-your-own-key (OpenAI/Claude/Gemini). Default decided by 4-way latency+quality benchmark, not preference. Groq optional extra.
- **Zero-cost mandate (Daxx, 2026-07-06): no monthly bill.** Daxx's own default = Parakeet V3 ASR + local cleanup (path A). BYO-key path stays in the product as an option (needed for the shareable exe), but Daxx's benchmark decision is among LOCAL models only unless a local model can't pass the quality gate — surface to Daxx before falling back to paid.
- **Cleanup model: Qwen3-4B (benchmark overturned the Llama-8B pick, 2026-07-06).** Llama-3.1-8B failed the golden rule on the T-battery (paraphrased T5, dropped "I think" in most prompt variants); Qwen3-4B = zero keep-words violations across all prompt versions, 90–300ms, fails only by under-editing (safe direction). Per register rule "default decided by benchmark, not preference." Known miss: won't rejoin ASR's choppy fragments (T1) — assigned to Qwen3-8B eval (downloaded) and ultimately the LoRA fine-tune on Daxx's regression pairs. Active prompt: v1.7 (C:\FlowSpeech\llm\cleanup_prompt_v17.txt, mirrored in docs/CLEANUP_PROMPT.md); sent as SYSTEM message, transcript as user (actions.rs modern path). **End state unchanged: engine embedded in FlowSpeech.exe** (docs/EMBED_PLAN.md); llama-server bench (C:\FlowSpeech\llm\) is Phase-A scaffolding only — NOT autostarted (Daxx: no helper); until embed lands, cleanup hotkey needs the bench server running; raw dictation unaffected.
- History: OFF by default (history_limit=0), never save audio recordings. Keep aggregate counters only: words, per-stage latency, estimated API cost per provider.
- Short utterances (<~6 words) skip the LLM (toggle).
- Offline/no-key fallback: rule-based filler stripper — dictation never hard-fails.
- Shareable exe: keep MIT license text, rename away from "Handy" (trademark), users bring their own key or use local mode.

## Working agreements (Daxx feedback, standing)
- Surface vendor/provider/cost decisions to Daxx before locking them — never bury them in a register.
- Explain installs before running them; system-level installs need his explicit OK (UAC clicks are his).
- BOT team gates run visibly (SAGE/PRODUCT/STRUCTURE speak), not silent-mode summaries.
- Plain language, short; 🟡 gold-marker takeaway lines for skimming.
- Effort: Medium default; High for hotkey-hook work, cleanup-prompt engineering, latency optimization.
- Sub-agents: Sonnet for well-specified code chunks, Haiku for mechanical sweeps, Fable for ambiguous/hard 20%.

## Build notes
- Build with `$env:CARGO_TARGET_DIR = "C:\h"` — keeps Rust artifacts out of OneDrive AND avoids the Windows 260-char path bug (MSB3491, see BUILD.md).
- Repo lives inside OneDrive (`Business Files\FlowSpeech\flowspeech`); if OneDrive file-locking ever breaks `bun install`/builds, pause sync during heavy operations.
- **Required build env (2026-07-06, first successful build):** load vcvars64 into the shell, then set `CMAKE_GENERATOR=Ninja` — the default VS generator hits a known MSBuild file-tracker bug (MSB6003/DirectoryNotFoundException, dotnet/msbuild#4831) in ggml's vulkan-shaders-gen nested configure. Ninja + CMake ship with VS Build Tools. Also put `%USERPROFILE%\.cargo\bin`, VS CMake/Ninja dirs, and `C:\VulkanSDK\1.4.350.0\Bin` on PATH (rustc/cmake are not on the system PATH). Vulkan SDK 1.4.350.0 installed machine-wide (winget, 2026-07-06); it bundles SPIRV-Headers config, so upstream CI's vcpkg step is NOT needed.
- Local-fork config changes (do not revert): tauri.conf.json has upstream's Azure `signCommand` removed and `createUpdaterArtifacts: false` — both were CI-only and break local bundling.
- Full working build command lives in the session transcript; canonical form:
  `vcvars64 → $env:CMAKE_GENERATOR="Ninja"; $env:VULKAN_SDK="C:\VulkanSDK\1.4.350.0"; $env:CARGO_TARGET_DIR="C:\h"; bun run tauri build`
- Artifacts land in `C:\h\release\` (handy.exe + ggml DLLs incl. ggml-vulkan.dll) and `C:\h\release\bundle\` (MSI + NSIS setup exe).
- **CRITICAL (2026-07-08): use `bun run tauri build`, NOT `cargo build`, for any exe whose UI you'll run.** `cargo build` produces a DEV-mode frontend that points the webview at the Vite dev server (localhost:1420) → "ERR_CONNECTION_REFUSED" blank window, no shortcuts, no paste. Only `tauri build` embeds the frontend + sets prod mode. For a portable/no-installer exe use `bun run tauri build --no-bundle` (compiles release exe with embedded frontend, skips the installer step that can't hold the 2.5GB model). Cost me hours of false "backend works" tests that never exercised the UI.
- **Embedded LLM: never init LlamaBackend on the startup/setup thread.** `LlamaBackend::init()` scans Vulkan devices (heavy) and blocked the main-window + shortcut + Enigo init when called during app setup → dead UI. cleanup.rs stores the model PATH at startup (cheap) and defers ALL llama.cpp init to the first `run()` on a spawn_blocking worker thread.

## Build phase status (update as you go)
- [x] Toolchain: Rust 1.96 + MSVC 14.44 + bun installed; repo cloned; VAD model downloaded
- [x] Moved to Business Files\FlowSpeech; renamed FlowSpeech
- [x] First build (`bun run tauri build`) — done 2026-07-06: GPU (Vulkan) build, MSI + NSIS installers in C:\h\release\bundle\
- [x] **Fork fix (2026-07-06): mic-stream self-heal** — always-on stream died permanently on device loss (BT profile switch / Wispr grabbing mic → "Recorder not available" until app restart; hit by Daxx in the wild, QA case B1/B2). try_start_recording now rebuilds the stream once and retries (managers/audio.rs). Manual B1 test (unplug mic mid-session, dictate) still owed.
- [x] **Fork fix (2026-07-06): audio-never-saved (QA H4 finding, now closed)** — root cause was NOT a missing feature: `RecordingRetentionPeriod::Never`/cleanup-by-count-0 already existed in managers/history.rs; `default_history_limit()` in settings.rs was just left at upstream's `5` instead of the locked `0`. Changed to 0 + comment. Verified: one test dictation swept all 6 accumulated WAVs (5 stale from testing + the new one) to zero, confirmed via log ("Cleaned up 6 old history entries by count") and directory listing. cleanup_by_count(0) runs synchronously in save_entry, so a WAV never survives past the same request that created it.
- [~] Verify stock Handy: hotkey → local ASR → paste — automated QA PASSED 2026-07-06 (17 tests, see docs/QA_TEST_PLAN.md §5): install + Parakeet V3 staged + end-to-end dictation works, release-to-text ≈0.45s, zero network, updater disabled. OPEN: Daxx's real-voice tests (accuracy, hotkey feel, daily paste targets). FINDING: stock saves WAV recordings (capped 5) — fix when applying history-off defaults.
- [x] Configure cleanup provider (local llama.cpp @ 11434, zero-cost path) + THE cleanup prompt v1.7 — wired into Ctrl+Shift+Space, verified end-to-end 2026-07-06. BYO-key path deferred (zero-cost mandate).
- [x] Benchmark → default = Qwen3-4B (see decision register; Llama-8B failed golden rule, Qwen3-8B eval pending for fragment-joining)
- [x] **EMBEDDED cleanup LLM in-process (2026-07-08)** — llama-cpp-2 0.1.139 linked into the app; the ggml-collision fear was unfounded (transcribe-cpp + llama.cpp each statically link their own ggml, link clean, GPU-coexist at runtime on the 5070 Ti). New: managers/cleanup.rs (global singleton, lazy load, greedy sampling, ChatML for Qwen); actions.rs `embedded` provider intercept (graceful raw-transcript fallback); settings default provider = `embedded`; lib.rs resolves bundled shards at startup. Requires LLVM/libclang (build-time, for bindgen) — installed machine-wide via winget. VERIFIED end-to-end in-process, no helper: "Embedded cleanup succeeded" ~800ms.
- [x] **Model bundling via GGUF sharding (2026-07-08)** — Windows installers (NSIS makensis + WiX light, both 32-bit) CANNOT hold the 2.5GB Qwen file OR a ~1.5GB+ total payload (mmap address-space limit). Solution: llama-gguf-split into 2 shards (<2GB each) at C:\FlowSpeech\llm\models\cleanup-shards\; llama.cpp auto-loads all shards from the first. Parakeet (640MB) bundled too. **Distribution = PORTABLE ZIP, not installer** (installers are physically out for this payload): C:\FlowSpeech\dist\FlowSpeech-portable.zip. Uses Handy's built-in portable mode (`portable` marker file → Data/ dir next to exe); Parakeet pre-placed in Data/models/, cleanup shards in resources/models/cleanup/, pre-configured settings (onboarding done, embedded cleanup, parakeet, history 0). Friend unzips + runs FlowSpeech.exe — zero download, zero helper, zero setup. VERIFIED: full pipeline runs from the portable dir.
- [ ] Ctrl+Win hotkey + Start-menu suppression
- [ ] Dictionary + per-app style profiles
- [ ] History-off defaults + cost counters
- [ ] Phase-0 regression suite vs Wispr → quality gate → cancel decision
