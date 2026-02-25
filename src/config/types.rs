use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub client: HashMap<String, ClientConfig>,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub repos: Vec<PathBuf>,
    #[serde(default = "default_hourly_rate")]
    pub hourly_rate: f64,
    #[serde(default = "default_currency")]
    pub currency: String,
    /// Cost per 1000 lines changed to offset LLM usage (default 0.0 = disabled)
    #[serde(default)]
    pub llm_cost_per_kloc: f64,
}

fn default_hourly_rate() -> f64 {
    0.0
}

fn default_currency() -> String {
    "USD".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Minutes of inactivity before starting a new session
    #[serde(default = "default_session_gap_minutes")]
    pub session_gap_minutes: u32,
    /// Minutes to add for the first commit of a session
    #[serde(default = "default_first_commit_minutes")]
    pub first_commit_minutes: u32,
    /// Do not compute sessions that cross weekends
    #[serde(default)]
    pub exclude_weekends: bool,
    /// Author emails/names to exclude (bots, CI systems)
    #[serde(default = "default_bot_authors")]
    pub bot_authors: Vec<String>,
    /// Enable volume-based time adjustment
    #[serde(default)]
    pub volume_adjustment: bool,
    /// Base minutes for volume bonus per commit (logarithmic scale)
    #[serde(default = "default_volume_factor")]
    pub volume_factor: f64,
    /// Lines-changed normalization divisor
    #[serde(default = "default_volume_scale")]
    pub volume_scale: f64,
}

fn default_session_gap_minutes() -> u32 {
    120
}

fn default_first_commit_minutes() -> u32 {
    30
}

fn default_bot_authors() -> Vec<String> {
    vec![
        "dependabot[bot]".to_string(),
        "github-actions[bot]".to_string(),
        "renovate[bot]".to_string(),
    ]
}

fn default_volume_factor() -> f64 {
    5.0
}

fn default_volume_scale() -> f64 {
    50.0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            session_gap_minutes: default_session_gap_minutes(),
            first_commit_minutes: default_first_commit_minutes(),
            exclude_weekends: false,
            bot_authors: default_bot_authors(),
            volume_adjustment: false,
            volume_factor: default_volume_factor(),
            volume_scale: default_volume_scale(),
        }
    }
}
