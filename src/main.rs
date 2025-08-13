use notify_rust::{Notification, Timeout};
use serde::Deserialize;
use serde_json::Value;
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
struct HookPayload {
    event: String,
    #[serde(default)]
    content: Option<Value>,
    #[serde(default)]
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read JSON from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    
    // Parse JSON payload
    let payload: HookPayload = serde_json::from_str(&buffer)?;
    
    // Determine notification based on event type
    let (title, body, sound) = match payload.event.as_str() {
        "Notification" => {
            // Claude needs approval
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
            ("Claude Needs Approval", body, "Glass")
        },
        "PreToolUse" => {
            // Claude is about to use a tool
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
                // Try to extract tool info from content
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
            ("Claude Tool Use", body, "Pop")
        },
        "Stop" => {
            // Claude has finished
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
            ("Claude Finished", body, "Hero")
        },
        _ => {
            // Unknown event - still notify but with generic message
            ("Claude Event", format!("Event: {}", payload.event), "Tink")
        }
    };
    
    // Send notification
    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(&body)
        .appname("Claude Code")
        .timeout(Timeout::Milliseconds(5000));
    
    // macOS specific: use terminal-notifier features via notify-rust
    #[cfg(target_os = "macos")]
    {
        notification.sound_name(sound);
    }
    
    notification.show()?;
    
    Ok(())
}
