---
name: vant-test
description: >
  Run all project tests. Runs cargo test for the Rust engine and xcodebuild test
  for Swift targets. Use when the user says "test", "check", "verify", or after
  implementing a feature or fixing a bug.
allowed-tools: Bash, Read, Glob, Grep
---

# Run Vant Tests

## Steps

1. **Run Rust tests**:
   ```bash
   cd /Users/alanguyen/Code/Others/vant && export PATH="$HOME/.cargo/bin:$PATH" && cd vant-engine && cargo test 2>&1
   ```

2. **Run Swift tests** (if Xcode project has tests):
   ```bash
   cd /Users/alanguyen/Code/Others/vant/vant-macos && xcodebuild test -scheme VantIME -configuration Debug 2>&1
   ```

3. **Report results**:
   - Number of tests passed/failed for each component
   - If any tests fail, show the failure details and suggest fixes
   - Update PROGRESS.md if relevant tasks are verified complete
