# FlowSpeech

**Free, private voice-to-text that runs 100% on your own computer.** No account, no subscription, nothing sent to the cloud. Press a shortcut, talk, and your words appear in whatever you're typing in — optionally cleaned up by a built-in AI that fixes punctuation and removes filler words.

---

## Download

**➡️ [Download FlowSpeech (Google Drive)](PASTE_YOUR_GOOGLE_DRIVE_LINK_HERE)** · ~3 GB zip · Windows 10/11 (64-bit)

Everything is included in the download — the speech models and the cleanup AI are already inside. Nothing else to install, nothing to download on first run.

## How to install (first time)

1. Download the ZIP from the link above.
2. Right-click it → **Extract All**. Unzip it somewhere normal like your **Desktop** or **Documents** (⚠️ not the Program Files folder).
3. Open the unzipped **FlowSpeech** folder and double-click **FlowSpeech.exe**.
4. The first time, Windows may show a blue **"Windows protected your PC"** box. This is normal for apps that don't come from the Microsoft Store, and it's safe. Click **More info**, then **Run anyway** — Windows won't ask again.
5. FlowSpeech starts in your system tray (bottom-right, near the clock).

## How to use

| Shortcut | What it does |
| --- | --- |
| **Hold `Ctrl + Space`** | Dictate — talk while holding, release to paste the text wherever your cursor is |
| **Hold `Ctrl + Shift + Space`** | Dictate **with AI cleanup** — same, but punctuation is fixed and filler words ("um", "uh") are removed |

Works in any text field — email, chat, docs, browser, code editor.

## What's under the hood

- **Speech recognition:** NVIDIA Parakeet V3, running locally on your machine (no internet needed).
- **AI cleanup:** a local language model (Qwen3-4B) runs entirely in the app — no API keys, no per-word charges, no data leaving your computer.
- **Private by default:** history off, audio recordings never saved.

## Requirements & notes

- **Windows 10 or 11, 64-bit.**
- **A modern GPU is recommended** (the AI cleanup is fast on a decent graphics card). On a weak or GPU-less machine, plain dictation still works fine; the AI cleanup will just be slower.
- **~4 GB free disk space** after unzipping.
- Unzip to a regular folder you can write to — not Program Files.

## Credits

FlowSpeech is built on [**Handy**](https://github.com/cjpais/Handy) by CJ Pais — an excellent open-source, offline speech-to-text app — under the MIT License. Huge thanks to CJ and the Handy contributors, and to the Whisper, Parakeet, ggml/llama.cpp, and Qwen projects that make local speech and language models possible.

Licensed under the MIT License. The "Handy" name and brand assets belong to their owner and are not used here; FlowSpeech is an independently branded personal fork.
