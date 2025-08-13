use notify_rust::{Notification, Timeout};
use serde::Deserialize;
use serde_json::Value;
use std::io::{self, Read};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct HookPayload {
    pub event: String,
    #[serde(default)]
    pub content: Option<Value>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_config")]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub testing: TestConfig,
    #[serde(default)]
    pub debug: DebugConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NotificationConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    #[serde(default = "default_sounds")]
    pub sounds: SoundConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SoundConfig {
    #[serde(default = "default_approval_sound")]
    pub approval: String,
    #[serde(default = "default_tool_sound")]
    pub tool_use: String,
    #[serde(default = "default_completion_sound")]
    pub completion: String,
    #[serde(default = "default_unknown_sound")]
    pub unknown: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TestConfig {
    #[serde(default)]
    pub send_notifications: bool,
    #[serde(default = "default_delay")]
    pub notification_delay: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DebugConfig {
    #[serde(default)]
    pub enabled: bool,
}

fn default_config() -> NotificationConfig {
    NotificationConfig {
        timeout: default_timeout(),
        sounds: default_sounds(),
    }
}

fn default_timeout() -> u32 { 5000 }
fn default_sounds() -> SoundConfig {
    SoundConfig {
        approval: default_approval_sound(),
        tool_use: default_tool_sound(),
        completion: default_completion_sound(),
        unknown: default_unknown_sound(),
    }
}
fn default_approval_sound() -> String { "Glass".to_string() }
fn default_tool_sound() -> String { "Pop".to_string() }
fn default_completion_sound() -> String { "Hero".to_string() }
fn default_unknown_sound() -> String { "Tink".to_string() }
fn default_delay() -> u64 { 1000 }

pub struct NotificationData {
    pub title: String,
    pub body: String,
    pub sound: String,
}

pub fn load_config() -> Config {
    if Path::new("config.toml").exists() {
        let contents = fs::read_to_string("config.toml")
            .expect("Failed to read config.toml");
        toml::from_str(&contents)
            .expect("Failed to parse config.toml")
    } else {
        Config {
            notifications: default_config(),
            testing: TestConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

pub fn process_hook_event(payload: &HookPayload, config: &Config) -> NotificationData {
    match payload.event.as_str() {
        "Notification" => {
            let body = if let Some(metadata) = &payload.metadata {
                if let Some(tool_name) = &metadata.tool_name {
                    format!("Claude needs approval to use: {}", tool_name)
                } else if let Some(message) = &metadata.message {
                    message.clone()
                } else {
                    "Claude needs your approval".to_string()
                }
            } else {
                "Claude needs your approval".to_string()
            };
            NotificationData {
                title: "Claude Needs Approval".to_string(),
                body,
                sound: config.notifications.sounds.approval.clone(),
            }
        },
        "PreToolUse" => {
            let body = if let Some(metadata) = &payload.metadata {
                match metadata.tool_name.as_deref() {
                    Some("Bash") => {
                        if let Some(command) = &metadata.command {
                            format!("Running: {}", command)
                        } else {
                            "Running bash command".to_string()
                        }
                    },
                    Some(tool) => format!("Using tool: {}", tool),
                    None => "Using tool".to_string()
                }
            } else if let Some(content) = &payload.content {
                if let Some(tool_name) = content.get("tool_name").and_then(|v| v.as_str()) {
                    if tool_name == "Bash" {
                        if let Some(params) = content.get("parameters") {
                            if let Some(cmd) = params.get("command").and_then(|v| v.as_str()) {
                                format!("Running: {}", cmd)
                            } else {
                                format!("Using tool: {}", tool_name)
                            }
                        } else {
                            format!("Using tool: {}", tool_name)
                        }
                    } else {
                        format!("Using tool: {}", tool_name)
                    }
                } else {
                    "Using tool".to_string()
                }
            } else {
                "Using tool".to_string()
            };
            NotificationData {
                title: "Claude Tool Use".to_string(),
                body,
                sound: config.notifications.sounds.tool_use.clone(),
            }
        },
        "Stop" => {
            let body = if let Some(metadata) = &payload.metadata {
                if let Some(description) = &metadata.description {
                    description.clone()
                } else if let Some(message) = &metadata.message {
                    message.clone()
                } else {
                    "Task completed".to_string()
                }
            } else {
                "Task completed".to_string()
            };
            NotificationData {
                title: "Claude Finished".to_string(),
                body,
                sound: config.notifications.sounds.completion.clone(),
            }
        },
        _ => {
            NotificationData {
                title: "Claude Event".to_string(),
                body: format!("Event: {}", payload.event),
                sound: config.notifications.sounds.unknown.clone(),
            }
        }
    }
}

pub fn send_notification(data: &NotificationData, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
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
    let notification_data = process_hook_event(&payload, &config);
    send_notification(&notification_data, &config)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_with_notification(name: &str, data: NotificationData, config: &Config) {
        
        println!("\nTest: {}", name);
        println!("  Title: {}", data.title);
        println!("  Body: {}", data.body);
        println!("  Sound: {}", data.sound);
        
        if config.testing.send_notifications {
            if let Err(e) = send_notification(&data, config) {
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
        };
        
        let result = process_hook_event(&payload, &config);
        assert_eq!(result.title, "Claude Needs Approval");
        assert_eq!(result.body, "Claude needs your approval");
        assert_eq!(result.sound, "Glass");
        
        test_with_notification("test_empty_metadata", result, &config);
    }
}