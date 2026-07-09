# FlowSpeech Regression Suite — FlowSpeech vs Wispr Flow

Real utterances from Daxx, captured on both apps while the Wispr subscription is active.
Quality gate (SAGE holds it): for each pair, Daxx prefers FlowSpeech's output or can't tell.
Golden rule for the cleanup stage: **fix, never rewrite; keep his words.**

Each case records: what was actually said (or read), what each app produced, and the diagnosis.
Re-run cases after every pipeline change (settings, model, cleanup prompt).

---

## Case 001 — 2026-07-06 · read-aloud passage · same mic, back-to-back

**Source text read aloud:**
> The app is live in your tray right now — hold Ctrl+Space and talk. What's left needs your actual voice: real accuracy … names you use daily.

**FlowSpeech (Parakeet V3, stock settings, BEFORE mic-warm fix):**
> Gap is live in your tray right now. Hold control space and talk. What's left needs your actual voice. Real accuracy. Names you use daily.

**Wispr Flow:**
> The app is live in your tray right now. Hold Control+Space to talk. What's left needs your actual voice, real accuracy, names you use daily.

**Diagnosis:**
1. `"Gap is"` ← `"The app is"`: **onset clipping** — mic stream opened on key-press, first ~250 ms lost. NOT a model accuracy issue.
   → Fix applied 2026-07-06: `always_on_microphone: true` + `extra_recording_buffer_ms: 150`. Awaiting re-test (Case 001b).
2. Fragmented punctuation (`". Real accuracy. Names"` vs Wispr's flowing commas) and formatting (`"control space"` vs `"Control+Space"`): **cleanup-LLM gap** — Wispr post-processes with an LLM; our cleanup stage isn't built yet. This case becomes a benchmark input for it.

**Verdict: Wispr better (expected at this stage). Re-test after each fix.**

## Case 001b — 2026-07-06: same passage re-read after mic-warm fix

**FlowSpeech (AFTER `always_on_microphone: true` + 150ms tail buffer):**
> The app is live. The app is live. The app is live. (repeated readings — all correct)

**Result: ✅ onset clipping RESOLVED.** First words now captured intact.
One stray utterance still garbled ("Yeah, slag" — likely "Yeah, it's live"): short-utterance
accuracy case → benchmark input for the cleanup/dictionary stage.
Remaining Wispr gap = punctuation flow + formatting = cleanup-LLM stage (not yet built).

---

## Case 002 — 2026-07-06 · Daxx's real voice · first live three-way comparison

**Spoken:** "Umm, I think I am going to give it a test now."

**Wispr Flow:**
> I think I'm going to give it a test now.

**FlowSpeech raw (Ctrl+Space, Parakeet V3 + mic-warm + self-heal build):**
> I think I'm going to give it a test now.

**FlowSpeech polished (Ctrl+Shift+Space, + Qwen3-4B cleanup v1.7):**
> I think I'm going to give it a test now.

**Verdict: ✅ THREE-WAY TIE = quality-gate PASS on this case.**
- Raw Parakeet handled the "Umm" itself; kept "I think" (the hedge Llama-8B kept deleting in benchmarks).
- Cleanup pass changed nothing it shouldn't — zero word drift.
- Caveat: short, clean sentence. Gate needs 10+ diverse cases before the cancel decision —
  next cases should stress: dollar amounts/dates, self-corrections mid-sentence, proper nouns
  (Kisi, AccessSync, client names), and long rambling utterances (30+ words) where fragment
  choppiness and punctuation flow actually diverge.

## Case 003 — 2026-07-06 · long ramble + proper nouns + self-correction

**Spoken (approx):** "Okay, first I'm gonna talk about Kisi, and then I'm gonna talk about Titleist, the golf company, and then I'm going to ramble on until tomorrow — no, Friday."

**Wispr Flow:**
> okay, first I'm going to talk about Kisi, and then we're going to talk about Tidalist, the golf company, and then I'm going to ramble on till tomorrow, Friday.

**FlowSpeech raw (Parakeet):**
> Okay, first I'm gonna talk about K C and then I'm gonna talk about Tyrus, the golf company, and then I'm going to ramble on until tomorrow. No, Friday.

**FlowSpeech polished:** ❌ INVALID — output was the literal placeholder "cleaned transcript here."

**Findings:**
1. **Cleanup pipeline bug (FIXED same night):** the custom provider uses the legacy `${output}`-substitution path; prompt v1.7 had no `${output}`, so the LLM never received the transcript and invented a placeholder. Fix: `${output}` restored to the stored prompt. ⚠️ This also invalidates Case 002's "polished" line (cleanup was broken then too — likely pasted raw). Re-run both.
2. **Proper nouns — both engines failed differently:** Wispr got "Kisi" ✅ but "Tidalist" ❌ and changed "I'm" → "we're" (a word ERROR — Wispr rewrites too). Parakeet: "K C" ❌ / "Tyrus" ❌ (log shows a second attempt heard "Key C"/"Hotelist"). → Added to FlowSpeech dictionary (custom_words): Kisi, Titleist, AccessSync, FlowSpeech, Wispr, Daxx.
3. **Register fidelity:** Parakeet kept Daxx's actual "gonna"; Wispr formalized to "going to." FlowSpeech's golden rule says keep his words — point to us, pending Daxx's preference.
4. **Self-correction:** raw kept "until tomorrow. No, Friday." (needs cleanup rule 5); Wispr mushed it to "till tomorrow, Friday" (ambiguous — arguably worse).

**Verdict: no valid FlowSpeech-polished sample; Wispr made 2 word-level errors of its own. RE-TEST after fixes.**

## Case 004 — 2026-07-06 · clean single take, cleanup-only (no Wispr sample)

**Verified single hold on Ctrl+Shift+Space, 24s, via app log (not just pasted text).**

**Raw (Parakeet, from log):**
> Okay, so you're saying that it's working? I think it's I guess maybe it is working. also just so you know that all the text I sent last time was me for a test for polish. So that was a polish test. Did you actually test that?

**Polished (Qwen3-4B, v1.8 prompt):**
> Okay, so you're saying that it's working? I think it's maybe working. Also, just so you know, all the text I sent last time was me for a test for polish. So that was a polish test. Did you actually test that?

**Diff:** collapsed a stacked double-hedge ("I guess" + "maybe it is") to one hedge + dropped "it is"; capitalization/comma/redundant-"that" cleanup. Both real hedges ("I think", "maybe") preserved.

**Verdict: ✅ PASS against "polish, don't rewrite."** One borderline spot (the "it is" drop goes slightly past pure punctuation) — flagged, not failed. No Wispr sample provided this round.

**Process note:** confirmed via handy.log timestamps, not the pasted text alone — this is now the standard verification method after Case 003's stitched-fragments incident.

## How to add a case

1. Dictate the same utterance into FlowSpeech and Wispr (same mic, same room, back to back).
2. Paste both outputs under a new `## Case NNN` with the date and what was actually said.
3. Note which you preferred and why (accuracy / punctuation / formatting / speed).
4. Good sources: real work emails, Slack replies, names you use daily, numbers/dates, self-corrections mid-sentence, filler-heavy casual speech.
