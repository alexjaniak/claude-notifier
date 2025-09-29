use std::env;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub terminal_app: Option<String>,
    pub window_id: Option<String>,
    pub session_id: Option<String>,
    pub project_dir: Option<String>,
    pub parent_pid: Option<u32>,
    pub claude_pid: Option<u32>,
    pub cwd: Option<String>,
}

impl TerminalInfo {
    pub fn detect() -> Self {
        let mut info = TerminalInfo {
            terminal_app: None,
            window_id: None,
            session_id: None,
            project_dir: None,
            parent_pid: None,
            claude_pid: None,
            cwd: None,
        };

        // Get CLAUDE_PROJECT_DIR if available
        info.project_dir = env::var("CLAUDE_PROJECT_DIR").ok();
        
        // Get current working directory
        info.cwd = env::current_dir().ok().and_then(|p| p.to_str().map(String::from));
        
        // Detect terminal app from environment variables
        info.terminal_app = detect_terminal_from_env();
        
        // Get parent process info
        if let Some(ppid) = get_parent_pid() {
            info.parent_pid = Some(ppid);
            
            // Try to get more info about the parent process
            if let Some(parent_info) = get_process_info(ppid) {
                // If we didn't detect terminal from env, try from parent process
                if info.terminal_app.is_none() {
                    info.terminal_app = detect_terminal_from_process(&parent_info);
                }
            }
        }
        
        // Try to get window ID on macOS
        #[cfg(target_os = "macos")]
        {
            info.window_id = get_macos_window_id();
        }
        
        info
    }
}

fn detect_terminal_from_env() -> Option<String> {
    // Check for Cursor first (it also sets TERM_PROGRAM=vscode)
    if let Ok(git_askpass) = env::var("GIT_ASKPASS") {
        if git_askpass.contains("Cursor.app") {
            return Some("Cursor".to_string());
        }
    }
    
    // Check for Cursor via other environment variables
    if env::var("CURSOR_TRACE_ID").is_ok() {
        return Some("Cursor".to_string());
    }
    
    // Check common terminal environment variables
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        return Some(match term_program.as_str() {
            "iTerm.app" => "iTerm2".to_string(),
            "Apple_Terminal" => "Terminal".to_string(),
            "vscode" => "VSCode".to_string(),
            "WarpTerminal" => "Warp".to_string(),
            "tmux" => "tmux".to_string(),
            other => other.to_string(),
        });
    }
    
    // Check for VS Code
    if env::var("VSCODE_INJECTION").is_ok() || env::var("VSCODE_PID").is_ok() {
        return Some("VSCode".to_string());
    }
    
    // Check for other terminals
    if let Ok(terminal) = env::var("TERMINAL_EMULATOR") {
        return Some(terminal);
    }
    
    // Check for Alacritty
    if env::var("ALACRITTY_SOCKET").is_ok() {
        return Some("Alacritty".to_string());
    }
    
    // Check for WezTerm
    if env::var("WEZTERM_PANE").is_ok() {
        return Some("WezTerm".to_string());
    }
    
    // Check for Kitty
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return Some("Kitty".to_string());
    }
    
    None
}

fn get_parent_pid() -> Option<u32> {
    let output = Command::new("ps")
        .args(&["-p", &std::process::id().to_string(), "-o", "ppid="])
        .output()
        .ok()?;
    
    let ppid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    ppid_str.parse().ok()
}

fn get_process_info(pid: u32) -> Option<String> {
    let output = Command::new("ps")
        .args(&["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()?;
    
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn detect_terminal_from_process(process_name: &str) -> Option<String> {
    let lower = process_name.to_lowercase();
    
    if lower.contains("cursor") {
        Some("Cursor".to_string())
    } else if lower.contains("terminal") {
        Some("Terminal".to_string())
    } else if lower.contains("iterm") {
        Some("iTerm2".to_string())
    } else if lower.contains("code") {
        Some("VSCode".to_string())
    } else if lower.contains("warp") {
        Some("Warp".to_string())
    } else if lower.contains("alacritty") {
        Some("Alacritty".to_string())
    } else if lower.contains("wezterm") {
        Some("WezTerm".to_string())
    } else if lower.contains("kitty") {
        Some("Kitty".to_string())
    } else if lower.contains("hyper") {
        Some("Hyper".to_string())
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn get_macos_window_id() -> Option<String> {
    // Try to get the frontmost window ID using AppleScript
    let script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            set frontWindowID to id of front window of application process frontApp
            return frontWindowID as string
        end tell
    "#;
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
fn get_macos_window_id() -> Option<String> {
    None
}

pub fn activate_terminal(info: &TerminalInfo) -> Result<(), String> {
    if let Some(ref app) = info.terminal_app {
        activate_terminal_app(app)
    } else {
        Err("No terminal app detected".to_string())
    }
}

#[cfg(target_os = "macos")]
fn activate_terminal_app(app_name: &str) -> Result<(), String> {
    // Map our internal names to actual app names for AppleScript
    let actual_app_name = match app_name {
        "Cursor" => "Cursor",
        "VSCode" => "Visual Studio Code",
        "iTerm2" => "iTerm",
        "Terminal" => "Terminal",
        "Warp" => "Warp",
        "Alacritty" => "Alacritty",
        "WezTerm" => "WezTerm",
        "Kitty" => "kitty",
        "Hyper" => "Hyper",
        other => other,
    };
    
    let script = format!(r#"tell application "{}" to activate"#, actual_app_name);
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| format!("Failed to run AppleScript: {}", e))?;
    
    if output.success() {
        Ok(())
    } else {
        Err(format!("Failed to activate {}", app_name))
    }
}