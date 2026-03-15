# Vant Progress

## Current Phase: 1 - Telex Engine [COMPLETE]
Started: 2026-03-15
Completed: 2026-03-15

### Tasks
- [x] `event.rs` — SyllableEvent enum (Composing, Committed, Reset, Passthrough)
- [x] `engine.rs` — TelexEngine with process_key(), force_commit(), reset()
- [x] Unit tests — 42 engine tests (diacritics, tones, syllables, deferred, backspace, commit triggers, reset, edge cases)
- [x] `ffi.rs` — VantEventType, VantKeyResult, VantEngine opaque type, 6 FFI functions
- [x] `lib.rs` — FFI integration tests (9 tests)
- [x] Generated `vant_engine.h` — all new C types and functions present
- [x] `make build` — full pipeline (Rust + Xcode) succeeds

### Phase 1 Milestone: ACHIEVED
51 tests pass. TelexEngine processes keystrokes, emits events, and is fully callable from C/Swift via FFI.

## Next Phase: 2 - Minimal IME
- InputMethodKit integration (VantInputController)
- Wire VantEngine FFI calls to IMK event handling
- Preedit (marked text) display from Composing events
- Text insertion from Committed events

## Completed Phases
- Phase 0: Scaffold (2026-03-15)
- Phase 1: Telex Engine (2026-03-15)

## Upcoming Phases
- Phase 3: N-gram Prediction (Tier 1 FST dictionary)
- Phase 4: RWKV Integration (Tier 2 inference, ghost text)
- Phase 5: Polish (code-switching, settings, personalization)
