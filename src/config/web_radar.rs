use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRadarConfig {
    pub enabled: bool,
    pub web_radar_api: String,
    pub web_radar_endpoint: String,
    pub lobby_link: String,
}
