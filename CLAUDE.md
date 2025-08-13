# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust-based notification system for Claude Code hooks that sends native OS notifications when Claude performs various actions (approval requests, tool usage, task completion).

## Essential Commands

### Build & Run
```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run with test JSON input
echo '{"event":"Notification","metadata":{"tool_name":"Bash"}}' | cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run all tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_notification_event

# Run tests with actual notifications (enable in config.toml first)
# Set [testing] send_notifications = true
cargo test

# List all available tests
cargo test -- --list
```

### Linting & Formatting
```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt --check

# Run clippy for linting
cargo clippy
```

## Architecture

### Core Flow
1. **main.rs:main()** - Entry point that reads JSON from stdin, loads config, and sends notification
2. **process_hook_event()** - Maps Claude Code hook events to notification data (title, body, sound)
3. **send_notification()** - Sends the actual OS notification using notify-rust

### Event Processing
The system handles these Claude Code hook events:
- `Notification` - Approval requests
- `PreToolUse` - Tool usage (special handling for Bash commands)
- `Stop` - Task completion
- Unknown events - Generic fallback

### Configuration System
- **config.toml** - User configuration (notification settings, test options, debug mode)
- **Config struct hierarchy** - Type-safe configuration with serde deserialization and defaults
- **.env** - Reserved for future secrets/API keys only

### Testing Architecture
- Tests use a static configuration loaded once per test run
- `test_with_notification()` helper optionally sends real notifications based on config
- Each test validates notification data structure and can trigger actual OS notifications

## Key Design Decisions

1. **Configuration over environment variables**: Uses TOML config files (Rust idiomatic) instead of .env for settings
2. **Graceful fallbacks**: Missing metadata fields fall back to sensible defaults
3. **Platform-specific features**: macOS sound support via `#[cfg(target_os = "macos")]`
4. **Test isolation**: Static config instance prevents repeated file reads during testing

## Development Setup

1. Copy config example: `cp config.toml.example config.toml`
2. Enable test notifications in config.toml if needed
3. Use `cargo test <test_name>` to test individual notification types