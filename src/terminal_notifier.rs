use std::process::Command;
use crate::types::{NotificationData, Config};

/// Check if terminal-notifier is available
pub fn is_available() -> bool {
    Command::new("which")
        .arg("terminal-notifier")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the path to terminal-notifier
fn get_terminal_notifier_path() -> Option<String> {
    Command::new("which")
        .arg("terminal-notifier")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Send notification using terminal-notifier with click action support
pub fn send_notification(
    data: &NotificationData, 
    config: &Config, 
    session_id: Option<String>
) -> Result<(), Box<dyn std::error::Error>> {
    let terminal_notifier = get_terminal_notifier_path()
        .ok_or("terminal-notifier not found")?;
    
    let mut cmd = Command::new(terminal_notifier);
    
    // Basic notification parameters
    cmd.arg("-title").arg(&data.title)
       .arg("-message").arg(&data.body)
       .arg("-sound").arg(&data.sound)
       .arg("-appIcon").arg("https://www.anthropic.com/favicon.ico");
    
    // Add click action if we have a session ID and click behavior is enabled
    if config.notifications.click_behavior.enabled {
        if let Some(sid) = session_id {
            // Get the path to our activate_session binary
            let activate_binary = get_activate_session_binary();
            
            if let Some(binary_path) = activate_binary {
                // Create the command to execute on click
                let execute_cmd = format!("{} {}", binary_path, sid);
                cmd.arg("-execute").arg(execute_cmd);
                
                // Add action button label
                cmd.arg("-actions").arg(&config.notifications.click_behavior.action_label);
            } else {
                // Fallback: just use osascript to show an alert
                let script = format!(
                    "osascript -e 'display dialog \"Session: {}\"'",
                    sid
                );
                cmd.arg("-execute").arg(script);
            }
        }
    }
    
    // Execute the command
    let output = cmd.output()?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("terminal-notifier failed: {}", error).into());
    }
    
    Ok(())
}

/// Get the path to the activate_session binary
fn get_activate_session_binary() -> Option<String> {
    // First, try to find it in the target directory (development)
    let dev_paths = vec![
        "target/debug/activate_session",
        "target/release/activate_session",
    ];
    
    for path in dev_paths {
        let full_path = std::env::current_dir().ok()?
            .join(path);
        if full_path.exists() {
            return Some(full_path.to_string_lossy().to_string());
        }
    }
    
    // Try to find it in the same directory as the current executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let activate_path = dir.join("activate_session");
            if activate_path.exists() {
                return Some(activate_path.to_string_lossy().to_string());
            }
        }
    }
    
    // Try to find it in PATH
    Command::new("which")
        .arg("activate_session")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}