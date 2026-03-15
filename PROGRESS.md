# Vant Progress

## Current Phase: 0 - Scaffold [COMPLETE]
Started: 2026-03-15
Completed: 2026-03-15

### Tasks
- [x] Git init + .gitignore + LICENSE + README
- [x] CLAUDE.md (project conventions)
- [x] Rust crate skeleton (vant-engine) — cargo build + test pass, vant_engine.h generated
- [x] Xcode project skeleton (vant-macos) — xcodebuild succeeds for VantIME + Vant targets
- [x] Build scripts + Makefile — `make build` works end-to-end
- [x] Custom skills + agents — 4 skills + 2 agents created
- [x] Vietnamese syllable data — 4,214 base syllables in vi_syllables.json

### Phase 0 Milestone: ACHIEVED
`cargo build` + `xcodebuild` both succeed. `make build` orchestrates the full pipeline.

## Next Phase: 1 - Telex Engine
- Wrap vi-rs in TelexEngine with process_key() API
- Implement SyllableEvent emission system
- Expose via C FFI (cbindgen auto-generates header)
- Unit tests for all Telex rules + deferred diacritics

## Upcoming Phases
- Phase 2: Minimal IME (InputMethodKit integration)
- Phase 3: N-gram Prediction (Tier 1 FST dictionary)
- Phase 4: RWKV Integration (Tier 2 inference, ghost text)
- Phase 5: Polish (code-switching, settings, personalization)

## Completed Phases
- Phase 0: Scaffold (2026-03-15)
