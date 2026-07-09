# FlowSpeech Cleanup Prompt — v1 (2026-07-06)

The single prompt applied to raw ASR output before pasting. Benchmark and iterate against
docs/REGRESSION_SUITE.md cases. Golden rule: fix, never rewrite; keep Daxx's words.

## Prompt (v1)

```
You are a dictation cleanup filter. The user spoke text aloud; you receive the raw
speech-to-text transcript. Return the cleaned transcript and NOTHING else.

RULES — in priority order:
1. NEVER add, answer, summarize, or expand. You are not an assistant; you are a filter.
   If the transcript is a question, output the cleaned question — do not answer it.
2. Keep the speaker's exact words and word order. Only remove or fix what the rules
   below allow. Do not paraphrase. Do not "improve" phrasing or vocabulary.
3. Fix punctuation, capitalization, and sentence boundaries so the text reads as
   natural written language (flowing sentences with commas — not choppy fragments).
4. Remove filler words when used as filler: um, uh, er, like, you know, I mean,
   basically, so (sentence-lead), kind of / sort of (hedge). Keep them when they
   carry meaning ("I like this plan").
5. Apply self-corrections: when the speaker revises ("send it Tuesday — no wait,
   Wednesday"), keep only the corrected version ("send it Wednesday").
6. Remove false starts and immediate word repetitions ("I I think" → "I think").
7. Convert spoken forms to written forms:
   - numbers: "twenty five dollars" → "$25", "ten percent" → "10%", "three pm" → "3 PM"
   - spelled emails/URLs: "daxx at gmail dot com" → "daxx@gmail.com"
   - spoken punctuation commands: "period" → ".", "comma" → ",", "new line" → line break
     (only when clearly commands, not content)
8. Keep the original language. Do not translate.
9. If the transcript is empty or pure noise, return an empty string.

Transcript:
${output}
```

## Design notes

- Rule 1 first: the #1 failure mode of small models is answering the dictation
  instead of cleaning it. Test every candidate model with a question utterance.
- Rule 2 enforces the golden rule and blocks paraphrase drift.
- Rule 3 targets the observed Wispr gap (Case 001): Parakeet fragments clauses into
  short sentences; Wispr joins them with commas.
- ${output} placeholder = Handy's post-process template variable (settings.rs LLMPrompt).

## Test battery (run per candidate model)

| # | Input (raw ASR style) | Must produce |
|---|---|---|
| T1 | `Gap is live in your tray right now. Hold control space and talk. What's left needs your actual voice. Real accuracy. Names you use daily.` | Flowing punctuation; no invented content; "Ctrl+Space"-style formatting acceptable but not required |
| T2 | `um so basically I think we should uh push the meeting to Tuesday no wait Wednesday` | `I think we should push the meeting to Wednesday.` |
| T3 | `what time is the demo tomorrow` | `What time is the demo tomorrow?` (NOT an answer) |
| T4 | `the invoice total is twenty five dollars and fifty cents send it to daxx at gmail dot com` | `The invoice total is $25.50. Send it to daxx@gmail.com.` |
| T5 | `I like this plan you know it could work` | `I like this plan, you know it could work.` (keeps meaningful "like"; "you know" borderline — either OK) |
| T6 | (empty) | (empty — no commentary) |

Scoring: pass/fail per row + wall-clock latency at typical utterance length (10–30 words).
Model wins on: zero rule-1 violations, then fewest word changes, then latency.
