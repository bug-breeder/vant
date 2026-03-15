.PHONY: setup build build-release test install uninstall clean header

SHELL := /bin/bash
PROJECT_DIR := $(shell pwd)
ENGINE_DIR := $(PROJECT_DIR)/vant-engine
MACOS_DIR := $(PROJECT_DIR)/vant-macos
SCRIPTS_DIR := $(PROJECT_DIR)/scripts

# Ensure cargo is in PATH
export PATH := $(HOME)/.cargo/bin:$(PATH)

## setup: Install development dependencies
setup:
	@echo "==> Checking Rust toolchain..."
	@which rustc > /dev/null 2>&1 || (echo "ERROR: Rust not installed. Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" && exit 1)
	@rustc --version
	@echo "==> Checking cbindgen..."
	@which cbindgen > /dev/null 2>&1 || cargo install cbindgen
	@echo "==> Checking xcodegen..."
	@which xcodegen > /dev/null 2>&1 || (echo "WARNING: xcodegen not installed. Run: brew install xcodegen")
	@echo "==> Setup complete"

## build: Build Rust engine (debug) + Xcode targets (debug)
build: build-engine-debug build-xcode-debug

## build-release: Build everything in release mode
build-release: build-engine-release build-xcode-release

## build-engine-debug: Build Rust engine in debug mode
build-engine-debug:
	@bash $(SCRIPTS_DIR)/build-rust.sh debug

## build-engine-release: Build Rust engine in release mode
build-engine-release:
	@bash $(SCRIPTS_DIR)/build-rust.sh release

## build-xcode-debug: Build Xcode targets in debug mode
build-xcode-debug:
	@echo "==> Building Xcode targets (Debug)..."
	cd $(MACOS_DIR) && xcodebuild -target VantIME -configuration Debug build 2>&1 | tail -5
	cd $(MACOS_DIR) && xcodebuild -target Vant -configuration Debug build 2>&1 | tail -5

## build-xcode-release: Build Xcode targets in release mode
build-xcode-release:
	@echo "==> Building Xcode targets (Release)..."
	cd $(MACOS_DIR) && xcodebuild -target VantIME -configuration Release build 2>&1 | tail -5
	cd $(MACOS_DIR) && xcodebuild -target Vant -configuration Release build 2>&1 | tail -5

## test: Run all tests
test:
	@echo "==> Running Rust tests..."
	cd $(ENGINE_DIR) && cargo test
	@echo "==> All tests passed"

## install: Build release and install IME
install:
	@bash $(SCRIPTS_DIR)/install.sh

## uninstall: Remove VantIME from Input Methods
uninstall:
	@echo "==> Uninstalling VantIME..."
	@killall VantIME 2>/dev/null || true
	@rm -rf "$(HOME)/Library/Input Methods/VantIME.app"
	@echo "==> VantIME uninstalled. Log out and back in to complete removal."

## clean: Clean all build artifacts
clean:
	@echo "==> Cleaning Rust build..."
	cd $(ENGINE_DIR) && cargo clean
	@echo "==> Cleaning Xcode build..."
	cd $(MACOS_DIR) && xcodebuild clean 2>/dev/null || true
	@rm -rf $(MACOS_DIR)/build
	@echo "==> Clean complete"

## header: Regenerate C header via cbindgen
header:
	@echo "==> Regenerating vant_engine.h..."
	cd $(ENGINE_DIR) && cargo build
	@echo "==> Header regenerated"

## xcode-project: Regenerate Xcode project from project.yml
xcode-project:
	@echo "==> Regenerating Xcode project..."
	cd $(MACOS_DIR) && xcodegen generate
	@echo "==> Xcode project regenerated"

## help: Show this help
help:
	@echo "Vant Build System"
	@echo ""
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /'
