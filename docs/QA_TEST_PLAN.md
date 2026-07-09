# FlowSpeech QA Test Plan — Stock Build Verification (Parakeet V3, Local-Only)

**Build under test:** FlowSpeech 0.9.0 (Handy fork), GPU build 2026-07-06, installed from `C:\h\release\bundle\nsis\Handy_0.9.0_x64-setup.exe`
**Config under test:** Parakeet V3 (local CPU ONNX model) — the zero-cost path. No API keys configured, no cloud services.
**Machine:** Windows 11 Home, RTX 5070 Ti (GPU only used by Whisper models, not Parakeet), 24-thread CPU.
**Checklist step covered:** "Verify stock Handy: hotkey → local ASR → paste."

🟡 **Zero-cost claim to verify: with Parakeet V3, dictation must work with the network cable pulled and never generate a bill. Only expected network traffic ever: one-time model download + the update checker (see K3 — disable it).**

---

## 0. Facts pulled from source (test against these, not assumptions)

| Behavior | Stock default | Where |
|---|---|---|
| Transcribe hotkey | `Ctrl+Space`, **push-to-talk (hold)** | settings.rs:751,804 |
| Post-process hotkey | `Ctrl+Shift+Space` | settings.rs:771 |
| Cancel key | `Escape` | settings.rs:796 |
| History kept | last **5** transcripts (local SQLite) | settings.rs:524 |
| Always-on microphone | off | settings.rs:815 |
| Paste mechanism | save clipboard → write text → send Ctrl+V → restore clipboard | clipboard.rs:15 |
| Updater endpoint | **upstream Handy GitHub** — accepting an update replaces our fork | tauri.conf.json:78 |
| Audio storage | recordings never saved to disk (verify in H4) | — |

---

## 1. Setup preconditions (once)

- [ ] P-1 Install from the NSIS setup exe. App appears in tray.
- [ ] P-2 Windows Settings → Privacy → Microphone: allow desktop apps.
- [ ] P-3 Settings → Models: download **Parakeet V3** (~478 MB, one-time). Select it as active model.
- [ ] P-4 Confirm mic input device in Settings → Audio is the one you actually use.
- [ ] P-5 Open Debug mode (`Ctrl+Shift+D`) once to find App Data dir — note it for H4/M3.

---

## 2. Smoke tests (P0 — if any fail, stop and report)

| # | Test | Pass criteria |
|---|---|---|
| S1 | Hold Ctrl+Space in Notepad, say "this is a test", release | Text appears in Notepad within ~2s of release |
| S2 | Repeat S1 five times in a row | 5/5 success, no stuck recordings |
| S3 | Overlay appears while holding, disappears after | Matches recording state |
| S4 | Quit from tray, relaunch | Settings and model selection persisted |
| S5 | **Airplane test:** disable Wi-Fi/Ethernet, repeat S1 | Works identically offline |

---

## 3. Functional tests (P1)

| # | Area | Test | Pass criteria |
|---|---|---|---|
| F1 | Toggle mode | Turn off push-to-talk; tap Ctrl+Space to start, tap to stop | Both transitions work; overlay tracks state |
| F2 | Cancel | Start recording, press Escape | No text pasted, app returns to idle |
| F3 | History | Dictate 7 distinct phrases; open history | Only last 5 kept; oldest 2 gone |
| F4 | Rebind | Change hotkey to something else and back | New binding works immediately, survives restart |
| F5 | Mic switch | Change input device in settings, dictate | Uses new device |
| F6 | Single instance | Launch the exe again while running | No second instance; window comes to front |
| F7 | CLI remote | `handy.exe --toggle-transcription` from PowerShell while running | Toggles recording on the running instance |
| F8 | Autostart | Enable autostart, reboot | App in tray after login; hotkey works without opening window |

---

## 4. Edge cases

### A. Speech content (P1)
| # | Test | Watch for |
|---|---|---|
| A1 | Hold hotkey, say nothing, release | No paste, no garbage, no crash (VAD should filter) |
| A2 | Ultra-short: "yes" (<1s hold) | Either correct text or clean no-op — never a hang |
| A3 | Long dictation: 3+ minutes continuous | Full text, no truncation, memory doesn't balloon |
| A4 | Numbers/formats: phone number, email address, URL, dollar amount, date | Check how Parakeet formats these — capture examples for the later cleanup-prompt work |
| A5 | Filler-heavy: "um, so, like, basically…" | Note raw output — this is the baseline the cleanup LLM must fix |
| A6 | Self-correction: "send it Tuesday — no wait, Wednesday" | Note raw output (Wispr handles this; capture for regression suite) |
| A7 | Proper nouns you use daily (names, product names, "Kisi", "AccessSync") | Log misses — feeds the dictionary feature later |
| A8 | Whisper-quiet speech, then loud speech | Both transcribe; no clipping weirdness |
| A9 | Fast speech (natural max pace) | Accuracy holds |
| A10 | TV/music in background while dictating | Only your speech transcribed (VAD + model robustness) |
| A11 | Mixed language sentence (Parakeet V3 auto-detects language) | Doesn't derail the English around it |

### B. Audio devices (P1 — this is where desktop apps usually break)
| # | Test | Watch for |
|---|---|---|
| B1 | Unplug/disconnect mic **mid-recording** | Graceful failure, no crash, recoverable next attempt |
| B2 | Bluetooth headset: dictate while it's also on a call app (profile switch A2DP↔HFP) | App follows or fails cleanly |
| B3 | Change Windows default mic while app is running | Uses the configured device, not confused |
| B4 | Start recording while another app holds the mic (Teams/Discord call) | Both work (Windows shares mics) or clean error |
| B5 | No mic present at all (disable in Device Manager) | Clean error message, no crash loop |

### C. Hotkey & interaction (P1)
| # | Test | Watch for |
|---|---|---|
| C1 | Double-tap and rapid re-press during active transcription | No queue corruption, no double-paste |
| C2 | Press hotkey again while previous utterance still transcribing | Defined behavior (queued or ignored) — document which |
| C3 | Very short hold-release (<200 ms) | No stuck "recording" state |
| C4 | **Ctrl+Space conflict:** try in VS Code (IntelliSense) and with an IME active | Document the collision — justifies our Ctrl+Win rebind later |
| C5 | Hotkey while FlowSpeech settings window itself is focused | Works; pastes into settings search box or no-ops cleanly |
| C6 | Hold hotkey, Alt+Tab to another window, release | Text lands in the window focused **at release** — confirm which |

### D. Paste targets (P0 for the ones you use daily)
| # | Target | Watch for |
|---|---|---|
| D1 | Notepad, VS Code, browser textarea (Gmail) | Baseline — must all work |
| D2 | Browser address bar | Paste lands, no autocomplete fight |
| D3 | PowerShell / terminal | Ctrl+V behavior differs in some terminals |
| D4 | Password field | Document behavior (paste often blocked by field) |
| D5 | **Elevated (admin) window** e.g. admin PowerShell | Expected FAIL — Windows blocks synthetic input to elevated apps from non-elevated ones. Confirm it fails *cleanly* |
| D6 | Slack/Discord/Teams message box | Rich-text editors sometimes mangle paste |
| D7 | Excel cell | Text enters cell, no formula weirdness |
| D8 | No text field focused (desktop) | Nothing catastrophic; clipboard still restored |
| D9 | **Clipboard restore:** copy an image, dictate text, then Ctrl+V manually | Your image is back on the clipboard (clipboard.rs promises restore) |
| D10 | Clipboard restore under repeat: dictate twice back-to-back fast | No lost/wrong clipboard content |

### E. System lifecycle (P2)
| # | Test | Watch for |
|---|---|---|
| E1 | Sleep → resume → dictate | Hotkey still registered, mic still works |
| E2 | Lock screen (Win+L) while recording | No crash; clean state after unlock |
| E3 | Quit from tray while recording | Exits cleanly, no orphan process (check Task Manager) |
| E4 | Dictate under heavy CPU load (run a build in background) | Slower but correct; note latency hit |
| E5 | Windows restart with autostart on | Recovers full function without manual steps |

### F. Model & offline (P1)
| # | Test | Watch for |
|---|---|---|
| M1 | Kill network mid-model-download | Clean error/resume, no corrupt state |
| M2 | Switch Parakeet → Whisper → Parakeet mid-session | Each switch takes effect; no restart needed (or documented) |
| M3 | Corrupt test: rename a file inside the Parakeet model dir | Clean error, re-download offered — restore after |
| M4 | Extended offline: full day of use with network off | Zero degradation (this IS the product promise) |

### G. Latency baseline (P0 — feeds the ship gate)
| # | Test | Method |
|---|---|---|
| G1 | 10 utterances of ~10 words: time from key-release to text visible | Phone stopwatch or screen recording; log all 10, compute median + worst |
| G2 | Same but 30+ word utterances | Parakeet claims ~5x realtime on mid CPU; your machine should beat that |
| G3 | First dictation after app launch (cold model load) | Expect one-time lag; log it separately — don't let it pollute G1 |

🟡 **Ship gate reference: median <1s, p95 <1.5s release-to-text. Stock numbers here are the baseline the cleanup-LLM stage must fit inside.**

### H. Zero-cost & privacy verification (P0 — the reason we're here)
| # | Test | Pass criteria |
|---|---|---|
| H1 | Task Manager → App history / Resource Monitor network tab during 10 dictations | Zero network bytes from handy.exe during transcription |
| H2 | No API keys ever entered; grep settings store for key material | Nothing to bill against — impossible to be charged |
| H3 | Confirm no account/signup anywhere in app | None exists |
| H4 | After dictating, search App Data dir for `.wav`/audio files | No audio recordings persisted |
| H5 | History file: confirm transcripts stored **locally only** (App Data SQLite) | Present locally, nowhere else |

### K. Known issues to verify / config actions (from code review)
| # | Item | Action |
|---|---|---|
| K1 | Updater points to **upstream Handy releases** — an accepted update would overwrite the fork with stock Handy | Test that we can decline; **then disable update checks in our fork (code change queued)** |
| K2 | `Ctrl+Space` conflicts (VS Code, IME) | Confirmed by C4 — resolved by the planned Ctrl+Win rebind |
| K3 | Elevated windows can't receive paste (D5) | OS limitation — document as known, decide later if "run FlowSpeech elevated" is worth offering |

---

## 5. Results log

### Session 1 — 2026-07-06, automated run (Claude, TTS-through-speakers method)

Method: app silent-installed per-user, Parakeet V3 pre-staged, recording toggled via CLI, speech synthesized via Windows TTS through speakers into the mic array, results verified from `%LOCALAPPDATA%\com.pais.handy\logs\handy.log`.

| Test ID | Result | Notes |
|---|---|---|
| P-1..P-5 | ✅ | Silent install to `%LOCALAPPDATA%\Handy`; mic consent verified Allow (registry); 3 active capture devices |
| S1 | ✅ | TTS variant: transcribed + pasted into Notepad in one pass |
| S2 | ✅ | 7 dictations total, 7/7 processed, zero stuck states |
| S4 | ✅ | Settings survived force-kill + relaunch, hotkeys re-registered |
| S5 | ⚠️ | Evidence-based pass: zero TCP connections during transcription (H1). Full airplane-mode test needs admin — left for Daxx |
| F2 | ✅ | Escape cancelled recording, clean return to idle, no paste |
| F3 | ✅ | 7 dictations → exactly 5 recordings kept (PreserveLimit rotation works) |
| F6 | ✅ | Second instance forwards args and exits; no duplicate process |
| F7 | ✅ | `--toggle-transcription` remote control works (used for all tests) |
| A1 | ✅ | Pure silence → `sample count: 0`, "skipping persistence", no paste, no crash |
| D1 | ✅ | Pasted into Notepad (`Text pasted successfully in ~213ms`) |
| D9/D10 | ✅ | Clipboard sentinel restored after paste, and untouched on empty run |
| G1 | ✅ | ASR 0.12–0.19s per 3–5s utterance (23–26x real-time, CPU). Release-to-text ≈ **0.4–0.5s** — beats the <1s gate 2x |
| G3 | ✅ | Cold model load 1452ms (one-time; model unloads after 5 min idle → expect this lag on first use) |
| H1 | ✅ | Zero network connections owned by handy.exe during dictation |
| H2 | ✅ | All 8 post-process API key slots empty; nothing to bill |
| H4 | ❌ | **Stock DOES save audio**: WAV per dictation in `%APPDATA%\com.pais.handy\recordings\`, capped at 5 (`recording_retention_period: PreserveLimit`). Contradicts README. FlowSpeech config must set history 0 + retention off |
| K1 | ✅ RESOLVED | Updater disabled 3 layers deep (endpoints removed, plugin unregistered, setting defaults false) — verified `update_checks_enabled: false` in fresh install |
| E3-lite | ✅ | Force-kill → 0 orphan processes → clean relaunch |

Accuracy note: TTS robot voice through speakers garbled transcripts (e.g. "the boss jobs over the late film") — this is the acoustic path, NOT a Parakeet verdict. Real-voice accuracy tests remain open.

### Remaining for Daxx (needs a human voice / his judgment)

- A2–A11 (real speech content: accuracy, fillers, self-corrections, proper nouns, numbers)
- C1–C6 (hold-to-talk feel, double-press, Ctrl+Space conflicts in VS Code)
- D2–D8 (his real daily paste targets)
- E1/E2 (sleep/resume, lock screen), full S5 airplane test
- G1/G2 stopwatch with real speech (expected ≈ automated numbers)

**Bug template:** Test ID · what happened · expected · reproducible? (y/n/sometimes) · overlay state at the time · anything in debug mode (`Ctrl+Shift+D`).

---

## 6. Exit criteria for this checklist step

- All P0 sections pass (Smoke, D1-targets-you-use, G latency logged, H zero-cost verified).
- No P1 crash-class failures (hangs, stuck recording state, clipboard loss).
- A5/A6/A7 raw outputs captured — these become the "before" side of the regression suite vs Wispr.
- K1 verified and updater disable queued as a code task.

**Sign-off unlocks:** cleanup-LLM setup (local Ollama path — still $0/month) → 4-way benchmark → Ctrl+Win hotkey work.
