# Claude Notifier

A Rust-based notification system for Claude Code hooks. Sends native OS notifications when Claude performs various actions.

## Features

- Notifications for approval requests
- Tool usage notifications (with special handling for Bash commands)
- Task completion notifications
- Support for all Claude Code hook events

## Setup

1. Copy the example configuration file:
```bash
cp config.toml.example config.toml
```

2. Edit `config.toml` to customize settings:
- Enable/disable test notifications
- Adjust notification timeout
- Customize notification sounds (macOS)
- Enable debug mode

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

Configure this tool as a hook in your Claude Code settings to receive notifications for various events.