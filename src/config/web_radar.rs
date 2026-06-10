use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRadarConfig {
    pub enabled: bool,
    pub web_radar_api: String,
    pub web_radar_endpoint: String,
    pub lobby_link: String,
}

impl Default for WebRadarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            web_radar_api: String::default(),
            web_radar_endpoint: String::default(),
            lobby_link: String::default(),
        }
    }
}