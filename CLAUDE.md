# Vant - AI Vietnamese Input Method

## Project Overview
macOS-first AI-powered Vietnamese Input Method. Rust core engine + Swift/InputMethodKit frontend.

## Architecture
- `vant-engine/` — Rust library compiled to `libvant_engine.a` (static lib) with auto-generated `vant_engine.h` C header via cbindgen
- `vant-macos/` — Xcode project with two targets:
  - **VantIME**: InputMethodKit bundle installed to `~/Library/Input Methods/`
  - **Vant**: SwiftUI settings/installer companion app

## Build Commands
```bash
make setup          # Check Rust toolchain, install cbindgen
make build          # Build Rust engine + Xcode targets (Debug)
make build-release  # Build Release config
make test           # cargo test + xcodebuild test
make install        # Build release + install IME
make uninstall      # Remove IME + kill process
make clean          # Clean all build artifacts
make header         # Regenerate vant_engine.h via cbindgen
```

## Code Conventions
- **Rust**: snake_case, all C FFI functions in `src/ffi.rs`, prefixed `vant_engine_`
- **Swift**: camelCase, InputMethodKit classes in VantIME/ target
- **FFI boundary**: cbindgen generates `vant_engine.h` from Rust — never edit the header manually

## Key Rules
- Every Telex transformation rule must have a corresponding unit test
- All C FFI functions go in `ffi.rs` — no FFI scattered across modules
- The main thread (Swift/IMK) must never block on AI inference
- Check PROGRESS.md at the start of every session

## Design Reference
See `vant-proposed-design.md` for full architecture, latency budgets, and feature specifications.
