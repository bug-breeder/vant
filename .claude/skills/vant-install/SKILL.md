---
name: vant-install
description: >
  Build a release version of VantIME and install it to ~/Library/Input Methods/.
  Kills any existing VantIME process first. Use when the user says "install",
  "deploy", "try it out", or wants to test the IME on their system.
disable-model-invocation: true
allowed-tools: Bash, Read
---

# Install Vant IME

## Steps

1. **Kill existing VantIME process** (if running):
   ```bash
   killall VantIME 2>/dev/null || true
   ```

2. **Build release**:
   ```bash
   cd /Users/alanguyen/Code/Others/vant && make build-release
   ```

3. **Copy to Input Methods**:
   ```bash
   cp -R /Users/alanguyen/Code/Others/vant/vant-macos/build/Release/VantIME.app ~/Library/Input\ Methods/
   ```

4. **Notify user**:
   - Tell them to log out and back in (or restart) for macOS to detect the new input method
   - Then go to System Settings > Keyboard > Input Sources > "+" > search for Vant
