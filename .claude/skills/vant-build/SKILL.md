---
name: vant-build
description: >
  Build the vant project end-to-end. Compiles the Rust engine to a static library,
  generates the C header via cbindgen, then builds Xcode targets. Use when the user
  says "build", "compile", or after making code changes to either Rust or Swift code.
allowed-tools: Bash, Read, Glob, Grep
---

# Build Vant Project

## Steps

1. **Build Rust engine**:
   ```bash
   cd /Users/alanguyen/Code/Others/vant && export PATH="$HOME/.cargo/bin:$PATH" && cd vant-engine && cargo build --release
   ```

2. **Verify C header was generated**:
   - Check that `vant-engine/vant_engine.h` exists and contains expected function declarations
   - If missing, run `cargo build` again (cbindgen runs in build.rs)

3. **Build Xcode targets** (if vant-macos/ exists):
   ```bash
   cd /Users/alanguyen/Code/Others/vant/vant-macos && xcodebuild -scheme VantIME -configuration Debug build
   ```

4. **Report results**: Show build status for both Rust and Swift components. If any step fails, show the error and suggest a fix.

## Alternative: Use Makefile
If the Makefile is set up, prefer:
```bash
cd /Users/alanguyen/Code/Others/vant && make build
```
