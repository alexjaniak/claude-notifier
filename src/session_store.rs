use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::terminal_detector::TerminalInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub terminal_info: TerminalInfo,
    pub created_at: u64,
    pub cwd: Option<String>,
    pub transcript_path: Option<String>,
}

pub struct SessionStore {
    base_dir: PathBuf,
}

impl SessionStore {
    pub fn new() -> Self {
        let base_dir = std::env::temp_dir().join("claude-notifier-sessions");
        fs::create_dir_all(&base_dir).ok();
        
        // Clean up old sessions on startup
        Self::cleanup_old_sessions(&base_dir);
        
        SessionStore { base_dir }
    }
    
    pub fn store_session(&self, session_id: &str, terminal_info: TerminalInfo, cwd: Option<String>, transcript_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let session_info = SessionInfo {
            session_id: session_id.to_string(),
            terminal_info,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            cwd,
            transcript_path,
        };
        
        let file_path = self.session_file_path(session_id);
        let json = serde_json::to_string_pretty(&session_info)?;
        fs::write(file_path, json)?;
        
        Ok(())
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let file_path = self.session_file_path(session_id);
        if !file_path.exists() {
            return None;
        }
        
        let contents = fs::read_to_string(file_path).ok()?;
        serde_json::from_str(&contents).ok()
    }
    
    fn session_file_path(&self, session_id: &str) -> PathBuf {
        // Sanitize session_id to be filesystem-safe
        let safe_id = session_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();
        
        self.base_dir.join(format!("{}.json", safe_id))
    }
    
    fn cleanup_old_sessions(base_dir: &Path) {
        // Remove sessions older than 24 hours
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() - 86400; // 24 hours in seconds
        
        if let Ok(entries) = fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                            if duration.as_secs() < cutoff_time {
                                fs::remove_file(entry.path()).ok();
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn list_sessions(&self) -> Vec<String> {
        let mut sessions = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        sessions.push(name.trim_end_matches(".json").to_string());
                    }
                }
            }
        }
        sessions
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}