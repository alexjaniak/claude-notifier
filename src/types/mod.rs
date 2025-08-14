use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct HookPayload {
    #[serde(alias = "hook_event_name")]
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub tool_name: Option<String>,
    pub command: Option<String>,
    pub description: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct NotificationData {
    pub title: String,
    pub body: String,
    pub sound: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_notification_config")]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub testing: TestConfig,
    #[serde(default)]
    pub debug: DebugConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            notifications: NotificationConfig::default(),
            testing: TestConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    #[serde(default = "default_sound_config")]
    pub sounds: SoundConfig,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            sounds: SoundConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            approval: default_approval_sound(),
            tool_use: default_tool_sound(),
            completion: default_completion_sound(),
            unknown: default_unknown_sound(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TestConfig {
    #[serde(default)]
    pub send_notifications: bool,
    #[serde(default = "default_delay")]
    pub notification_delay: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DebugConfig {
    #[serde(default)]
    pub enabled: bool,
}

// Default value functions
fn default_notification_config() -> NotificationConfig {
    NotificationConfig::default()
}

fn default_timeout() -> u32 { 5000 }

fn default_sound_config() -> SoundConfig {
    SoundConfig::default()
}

fn default_approval_sound() -> String { "Glass".to_string() }
fn default_tool_sound() -> String { "Pop".to_string() }
fn default_completion_sound() -> String { "Hero".to_string() }
fn default_unknown_sound() -> String { "Tink".to_string() }
fn default_delay() -> u64 { 1000 }