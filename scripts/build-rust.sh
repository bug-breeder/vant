#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ENGINE_DIR="$PROJECT_DIR/vant-engine"
CONFIG="${1:-release}"

export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Building vant-engine ($CONFIG)..."

cd "$ENGINE_DIR"

if [ "$CONFIG" = "release" ]; then
    CARGO_FLAG="--release"
    TARGET_DIR="target/release"
else
    CARGO_FLAG=""
    TARGET_DIR="target/debug"
fi

# Build for host architecture
cargo build $CARGO_FLAG

# Verify outputs
if [ -f "$TARGET_DIR/libvant_engine.a" ]; then
    echo "==> Static library: $TARGET_DIR/libvant_engine.a"
    ls -lh "$TARGET_DIR/libvant_engine.a"
else
    echo "ERROR: libvant_engine.a not found in $TARGET_DIR"
    exit 1
fi

if [ -f "vant_engine.h" ]; then
    echo "==> C header: vant_engine.h"
else
    echo "ERROR: vant_engine.h not generated"
    exit 1
fi

echo "==> Rust build complete ($CONFIG)"
