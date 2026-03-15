# Vant - AI-Powered Vietnamese Input Method

An intelligent Vietnamese input method for macOS that combines traditional Telex/VNI input with RWKV-7 neural language model predictions.

## Architecture

- **vant-engine/** — Rust core library: Telex/VNI processing, n-gram dictionary, RWKV inference coordination
- **vant-macos/** — Swift/InputMethodKit frontend: macOS IME integration, ghost text UI, settings app

## Building

```bash
make setup    # Install dependencies (Rust, cbindgen)
make build    # Build Rust engine + Xcode targets
make install  # Install IME to ~/Library/Input Methods/
make test     # Run all tests
```

## Requirements

- macOS 13.0+
- Rust 1.70+
- Xcode 15+

## License

MIT
