# Claude Notifier

A Rust-based notification system for Claude Code hooks. Sends native OS notifications when Claude performs various actions, with smart terminal detection and click-to-focus functionality.

## Features

- 🔔 **Native OS Notifications** for Claude Code events:
  - Approval requests
  - Tool usage (with special handling for Bash commands)
  - Task completion
  - All other Claude Code hook events
- 🖥️ **Smart Terminal Detection**: Automatically identifies your terminal/IDE
- 🎯 **Click-to-Focus** (macOS): Action buttons to jump back to the originating terminal
- 📊 **Session Management**: Tracks multiple concurrent Claude Code sessions
- ⚙️ **Highly Configurable**: Customize notifications, sounds, and behaviors

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/claude-notifier.git
cd claude-notifier

# Build the project
cargo build --release

# The binary will be in target/release/claude-notifier
```

## Setup

1. Copy the example configuration file:
```bash
cp config.toml.example config.toml
```

2. Edit `config.toml` to customize settings:
- Enable/disable notifications per event type
- Adjust notification timeout
- Customize notification sounds (macOS)
- Configure click-to-focus behavior
- Enable debug mode for troubleshooting

3. Configure as a Claude Code hook (see Usage section)

## Testing

### Run all tests (without notifications)
```bash
cargo test
```

### Run all tests with actual notifications
Enable in `config.toml`:
```toml
[testing]
send_notifications = true
```
Then run:
```bash
cargo test
```

### Run a specific test with notification
```bash
cargo test test_notification_event
```

### Available tests
- `test_notification_event` - Claude needs approval notification
- `test_notification_with_message` - Custom approval message
- `test_pre_tool_use_bash` - Bash command execution
- `test_pre_tool_use_other_tool` - Other tool usage
- `test_pre_tool_use_from_content` - Tool use from content field
- `test_stop_event` - Task completion with description
- `test_stop_with_message` - Task completion with message
- `test_unknown_event` - Unknown event handling
- `test_empty_metadata` - Fallback for empty metadata

## Usage with Claude Code

### Configure as a Hook

Add to your Claude Code settings (`.claude/settings.local.json`):

```json
{
  "hooks": {
    "preToolUse": "echo '$HOOK_PAYLOAD' | /path/to/claude-notifier",
    "postToolUse": "echo '$HOOK_PAYLOAD' | /path/to/claude-notifier",
    "notification": "echo '$HOOK_PAYLOAD' | /path/to/claude-notifier",
    "stop": "echo '$HOOK_PAYLOAD' | /path/to/claude-notifier"
  }
}
```

Once configured, you'll receive native OS notifications for:
- 🟡 **Approval Requests**: When Claude needs your permission
- 🔧 **Tool Usage**: When Claude runs commands or uses tools
- ✅ **Task Completion**: When Claude finishes tasks
- 📢 **Other Events**: Any other Claude Code hook events

## Session Management

The notifier automatically detects and stores information about each Claude Code session:
- Terminal application (VS Code, iTerm2, Terminal, etc.)
- Working directory
- Session ID from Claude Code
- Parent process information

### Click-to-Focus Feature

When notifications appear, you can click the action button to instantly return to the terminal where Claude is running.

#### Configuration

Customize the click behavior in `config.toml`:
```toml
[notifications.click_behavior]
enabled = true  # Enable/disable action buttons
action_label = "Go to Terminal"  # Customize button text
```

#### Manual Session Activation

You can also manually activate a terminal for a specific session:
```bash
# List all sessions and choose one interactively
cargo run --bin activate_session

# Activate a specific session directly
cargo run --bin activate_session <session_id>
```

### Supported Terminals/IDEs

The notifier can detect and activate:
- **Cursor** - AI-powered IDE
- **Visual Studio Code**
- **iTerm2**
- **Terminal.app**
- **Warp**
- **Alacritty**
- **WezTerm**
- **Kitty**
- **Hyper**

## Architecture

The project is organized into modular components:

- **main.rs** - Entry point and event processing
- **terminal_detector** - Identifies the running terminal/IDE
- **terminal_notifier** - Handles OS notifications with platform-specific features
- **session_store** - Manages persistent session data
- **types** - Shared type definitions and configuration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Your license here]