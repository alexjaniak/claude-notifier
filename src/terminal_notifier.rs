use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use crate::types::{NotificationData, Config};
use include_dir::{include_dir, Dir};

// Embed the terminal-notifier.app at compile time
static TERMINAL_NOTIFIER_APP: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/terminal-notifier.app");

/// Get the path to terminal-notifier, extracting bundled version if needed
pub fn get_terminal_notifier_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Use a consistent cache location
    let cache_dir = dirs::cache_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("claude-notifier");

    let app_path = cache_dir.join("terminal-notifier.app");
    let binary_path = app_path.join("Contents/MacOS/terminal-notifier");

    // Check if already extracted and still valid
    if binary_path.exists() {
        // Verify it's executable
        if let Ok(metadata) = fs::metadata(&binary_path) {
            if metadata.is_file() {
                return Ok(binary_path);
            }
        }
    }

    // Extract the bundled app
    extract_bundled_app(&cache_dir)?;

    // Ensure binary is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    Ok(binary_path)
}

/// Extract the bundled terminal-notifier.app to the cache directory
fn extract_bundled_app(cache_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(cache_dir)?;

    let app_path = cache_dir.join("terminal-notifier.app");

    // Remove old version if it exists
    if app_path.exists() {
        fs::remove_dir_all(&app_path)?;
    }

    // Extract all files from the embedded directory
    TERMINAL_NOTIFIER_APP.extract(&app_path)?;

    Ok(())
}

/// Send notification using bundled terminal-notifier
pub fn send_notification(
    data: &NotificationData,
    config: &Config,
    session_id: Option<String>
) -> Result<(), Box<dyn std::error::Error>> {
    let terminal_notifier = get_terminal_notifier_path()?;

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

// Simple dirs module for cache directory
mod dirs {
    use std::path::PathBuf;
    
    pub fn cache_dir() -> Option<PathBuf> {
        home_dir().map(|h| h.join("Library/Caches"))
    }

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}