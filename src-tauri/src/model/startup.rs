use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StartupPromptState {
    #[serde(default)]
    pub last_seen_version: String,
    #[serde(default)]
    pub launch_count: u32,
    #[serde(default)]
    pub last_donation_prompt_launch: u32,
}
