pub mod types;
pub mod terminal_detector;
pub mod session_store;
pub mod terminal_notifier;

use types::{Config, HookPayload, NotificationData};

pub fn process_hook_event(payload: &HookPayload, config: &Config) -> NotificationData {
    match payload.event.as_str() {
        "Notification" => {
            let body = if let Some(metadata) = &payload.metadata {
                if let Some(tool_name) = &metadata.tool_name {
                    format!("Claude needs approval to use: {tool_name}")
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
                match (&metadata.tool_name, &metadata.command) {
                    (Some(tool_name), Some(command)) if tool_name == "Bash" => {
                        format!("Running: {command}")
                    },
                    (Some(tool_name), _) => {
                        format!("Using tool: {tool_name}")
                    },
                    _ => "Using tool".to_string(),
                }
            } else if let Some(tool_name) = &payload.tool_name {
                // Check for tool_name at top level (new format)
                if tool_name == "Bash" {
                    if let Some(tool_input) = &payload.tool_input {
                        if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
                            format!("Running: {command}")
                        } else {
                            format!("Using tool: {tool_name}")
                        }
                    } else {
                        format!("Using tool: {tool_name}")
                    }
                } else {
                    format!("Using tool: {tool_name}")
                }
            } else if let Some(content) = &payload.content {
                if let Some(tool_name) = content.get("tool_name").and_then(|v| v.as_str()) {
                    if tool_name == "Bash" {
                        if let Some(params) = content.get("parameters") {
                            if let Some(cmd) = params.get("command").and_then(|v| v.as_str()) {
                                format!("Running: {cmd}")
                            } else {
                                format!("Using tool: {tool_name}")
                            }
                        } else {
                            format!("Using tool: {tool_name}")
                        }
                    } else {
                        format!("Using tool: {tool_name}")
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
                if let Some(desc) = &metadata.description {
                    desc.clone()
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
        _ => NotificationData {
            title: "Claude Event".to_string(),
            body: format!("Event: {}", payload.event),
            sound: config.notifications.sounds.unknown.clone(),
        },
    }
}