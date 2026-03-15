# Vant Progress

## Current Phase: 2.5 - Tone Placement Fix [COMPLETE]
Started: 2026-03-15
Completed: 2026-03-15

### Tasks
- [x] `tone.rs` — Vietnamese syllable parser (initial consonant, vowels, final consonant)
- [x] `tone.rs` — Tone detection/manipulation using vi crate utilities
- [x] `tone.rs` — Tone position algorithm ported from xkey/Unikey rules
- [x] `tone.rs` — `relocate_tone()` post-processor moves tone to correct vowel
- [x] `engine.rs` — Wired `relocate_tone()` into `recompose()`
- [x] Tests — 80 total (51 original + 6 tone relocation + 23 tone module unit tests)
- [x] `make build` — full pipeline (Rust + Xcode) succeeds

### Phase 2.5 Milestone: ACHIEVED
Fixed tone placement bugs where typing tone before final consonant put the mark on the wrong vowel (e.g., "Vịêt" → "Việt", "xúât" → "xuất"). Post-processing approach: vi crate handles diacritics, new tone.rs relocates misplaced tones using xkey-derived rules.

## Completed Phases
- Phase 0: Scaffold (2026-03-15)
- Phase 1: Telex Engine (2026-03-15)
- Phase 2: Minimal IME (2026-03-15)
- Phase 2.5: Tone Placement Fix (2026-03-15)

## Next Phase: 3 - N-gram Prediction
- Tier 1 FST dictionary

## Upcoming Phases
- Phase 4: RWKV Integration (Tier 2 inference, ghost text)
- Phase 5: Polish (code-switching, settings, personalization)
