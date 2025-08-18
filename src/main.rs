use claude_notifier::types::{Config, HookPayload, NotificationData};
use claude_notifier::{process_hook_event, terminal_detector::TerminalInfo, session_store::SessionStore};
use notify_rust::{Notification, Timeout};
use std::io::{self, Read};
use std::fs;
use std::path::Path;

pub fn load_config() -> Config {
    if Path::new("config.toml").exists() {
        let contents = fs::read_to_string("config.toml")
            .expect("Failed to read config.toml");
        toml::from_str(&contents)
            .expect("Failed to parse config.toml")
    } else {
        Config::default()
    }
}

pub fn send_notification(data: &NotificationData, config: &Config, session_id: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    // On macOS, prefer terminal-notifier if available for click support
    #[cfg(target_os = "macos")]
    {
        if claude_notifier::terminal_notifier::is_available() {
            if config.debug.enabled {
                eprintln!("Debug: Using terminal-notifier for notification");
            }
            return claude_notifier::terminal_notifier::send_notification(data, config, session_id);
        }
    }
    
    // Fallback to notify-rust
    let mut notification = Notification::new();
    notification
        .summary(&data.title)
        .body(&data.body)
        .appname("Claude Code")
        .timeout(Timeout::Milliseconds(config.notifications.timeout));
    
    #[cfg(target_os = "macos")]
    {
        notification.sound_name(&data.sound);
    }
    
    // Add action button if enabled and we have a session ID
    // Note: On macOS with notify-rust, actions are displayed but we can't detect clicks
    if config.notifications.click_behavior.enabled {
        if let Some(sid) = session_id.clone() {
            notification.action("open_terminal", &config.notifications.click_behavior.action_label);
            
            // Store session ID in notification subtitle for debugging
            if config.debug.enabled {
                notification.subtitle(&format!("Session: {}", &sid[..8.min(sid.len())]));
            }
        }
    }
    
    notification.show()?;
    
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config();
    
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    
    if config.debug.enabled {
        eprintln!("Debug: Received payload:\n{}", buffer);
    }
    
    let payload: HookPayload = serde_json::from_str(&buffer)?;
    
    // Store session info if we have a session ID
    if let Some(ref session_id) = payload.session_id {
        let store = SessionStore::new();
        let terminal_info = TerminalInfo::detect();
        
        if config.debug.enabled {
            eprintln!("Debug: Session ID: {}", session_id);
            eprintln!("Debug: Terminal detected: {:?}", terminal_info.terminal_app);
        }
        
        // Store the session with terminal info
        store.store_session(
            session_id,
            terminal_info,
            payload.cwd.clone(),
            payload.transcript_path.clone()
        ).ok();
    }
    
    let notification_data = process_hook_event(&payload, &config);
    send_notification(&notification_data, &config, payload.session_id)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use claude_notifier::types::Metadata;
    use serde_json::json;

    fn test_with_notification(name: &str, data: NotificationData, config: &Config) {
        
        println!("\nTest: {}", name);
        println!("  Title: {}", data.title);
        println!("  Body: {}", data.body);
        println!("  Sound: {}", data.sound);
        
        if config.testing.send_notifications {
            // Use a test session ID for testing
            let test_session_id = Some(format!("test-session-{}", name));
            if let Err(e) = send_notification(&data, config, test_session_id) {
                eprintln!("Failed to send notification: {}", e);
            } else {
                println!("  ✓ Notification sent!");
                std::thread::sleep(std::time::Duration::from_millis(config.testing.notification_delay));
            }
        }
    }

    #[test]
    fn test_notification_event() {
        let config = load_config();
        let payload = HookPayload {
            event: "Notification".to_string(),
            content: None,
            metadata: Some(Metadata {
                tool_name: Some("Bash".to_string()),
                command: None,
                description: None,
                message: None,
            }),
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Needs Approval");
        assert_eq!(result.body, "Claude needs approval to use: Bash");
        assert_eq!(result.sound, "Glass");
        
        test_with_notification("test_notification_event", result, &config);
    }

    #[test]
    fn test_notification_with_message() {
        let config = load_config();
        let payload = HookPayload {
            event: "Notification".to_string(),
            content: None,
            metadata: Some(Metadata {
                tool_name: None,
                command: None,
                description: None,
                message: Some("Custom approval message".to_string()),
            }),
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Needs Approval");
        assert_eq!(result.body, "Custom approval message");
        assert_eq!(result.sound, "Glass");
        
        test_with_notification("test_notification_with_message", result, &config);
    }

    #[test]
    fn test_pre_tool_use_bash() {
        let config = load_config();
        let payload = HookPayload {
            event: "PreToolUse".to_string(),
            content: None,
            metadata: Some(Metadata {
                tool_name: Some("Bash".to_string()),
                command: Some("ls -la".to_string()),
                description: None,
                message: None,
            }),
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Tool Use");
        assert_eq!(result.body, "Running: ls -la");
        assert_eq!(result.sound, "Pop");
        
        test_with_notification("test_pre_tool_use_bash", result, &config);
    }

    #[test]
    fn test_pre_tool_use_other_tool() {
        let config = load_config();
        let payload = HookPayload {
            event: "PreToolUse".to_string(),
            content: None,
            metadata: Some(Metadata {
                tool_name: Some("Read".to_string()),
                command: None,
                description: None,
                message: None,
            }),
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Tool Use");
        assert_eq!(result.body, "Using tool: Read");
        assert_eq!(result.sound, "Pop");
        
        test_with_notification("test_pre_tool_use_other_tool", result, &config);
    }

    #[test]
    fn test_pre_tool_use_from_content() {
        let config = load_config();
        let content = json!({
            "tool_name": "Bash",
            "parameters": {
                "command": "npm test"
            }
        });
        
        let payload = HookPayload {
            event: "PreToolUse".to_string(),
            content: Some(content),
            metadata: None,
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Tool Use");
        assert_eq!(result.body, "Running: npm test");
        assert_eq!(result.sound, "Pop");
        
        test_with_notification("test_pre_tool_use_from_content", result, &config);
    }

    #[test]
    fn test_stop_event() {
        let config = load_config();
        let payload = HookPayload {
            event: "Stop".to_string(),
            content: None,
            metadata: Some(Metadata {
                tool_name: None,
                command: None,
                description: Some("All tests passed successfully".to_string()),
                message: None,
            }),
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Finished");
        assert_eq!(result.body, "All tests passed successfully");
        assert_eq!(result.sound, "Hero");
        
        test_with_notification("test_stop_event", result, &config);
    }

    #[test]
    fn test_stop_with_message() {
        let config = load_config();
        let payload = HookPayload {
            event: "Stop".to_string(),
            content: None,
            metadata: Some(Metadata {
                tool_name: None,
                command: None,
                description: None,
                message: Some("Build completed".to_string()),
            }),
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Finished");
        assert_eq!(result.body, "Build completed");
        assert_eq!(result.sound, "Hero");
        
        test_with_notification("test_stop_with_message", result, &config);
    }

    #[test]
    fn test_unknown_event() {
        let config = load_config();
        let payload = HookPayload {
            event: "UnknownEvent".to_string(),
            content: None,
            metadata: None,
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Event");
        assert_eq!(result.body, "Event: UnknownEvent");
        assert_eq!(result.sound, "Tink");
        
        test_with_notification("test_unknown_event", result, &config);
    }

    #[test]
    fn test_empty_metadata() {
        let config = load_config();
        let payload = HookPayload {
            event: "Notification".to_string(),
            content: None,
            metadata: None,
            tool_name: None,
            tool_input: None,
            session_id: Some("test-session".to_string()),
            transcript_path: None,
            cwd: None,
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Needs Approval");
        assert_eq!(result.body, "Claude needs your approval");
        assert_eq!(result.sound, "Glass");
        
        test_with_notification("test_empty_metadata", result, &config);
    }
}