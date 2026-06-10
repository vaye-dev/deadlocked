use serde_json::json;
use std::sync::{Once, OnceLock};

use crate::config::web_radar::WebRadarConfig;
use crate::{
    config::Config,
    constants::cs2::{TEAM_CT, TEAM_T},
    cs2::{CS2, entity::player::Player},
};

use crate::websocket::WebSocketClient;

static WEBRADAR: OnceLock<WebSocketClient> = OnceLock::new();
static WEBRADAR_INIT: Once = Once::new();
static LOBBY_URL: OnceLock<String> = OnceLock::new();

impl CS2 {
    pub fn web_radar(&mut self, config: &Config) {
        let config = self.web_radar_config(config);

        if !config.enabled
            || config.web_radar_api.is_empty()
            || config.web_radar_endpoint.is_empty()
        {
            return;
        }

        WEBRADAR_INIT.call_once(|| {
            self.initialize_web_radar(
                config.web_radar_endpoint.clone(),
                config.web_radar_api.clone(),
            );
        });

        let Some(ws) = WEBRADAR.get() else {
            return;
        };

        let map = self.current_map();
        let players: Vec<Player> = self.players.clone();

        let player_data: Vec<_> = players
            .iter()
            .filter_map(|p| {
                let team = match p.team(self) {
                    TEAM_T => "T",
                    TEAM_CT => "CT",
                    _ => return None,
                };
                let pos = p.position(self);
                Some(json!({
                    "name": p.name(self),
                    "team": team,
                    "hp": p.health(self),
                    "x": pos.x,
                    "y": pos.y,
                    "weapon": p.weapon_name(self),
                }))
            })
            .collect();

        let _ = ws.send(json!({
            "cmd": "update",
            "data": {
                "map": map,
                "players": player_data,
            },
            "id": "u"
        }));
    }

    pub fn web_radar_lobby_url() -> Option<String> {
        LOBBY_URL.get().cloned()
    }

    fn initialize_web_radar(&mut self, endpoint: String, api_key: String) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            rt.block_on(async move {
                let client = match WebSocketClient::connect(&endpoint, &api_key).await {
                    Ok(client) => client,
                    Err(err) => {
                        println!("Connect failed: {}", err);
                        return;
                    }
                };

                if let Some(msg) = client.read().await {
                    println!("Authenticated as: {}", msg["name"]);
                }

                let _ = client.send(json!({"cmd": "lobby", "id": "open"}));

                if let Some(msg) = client.read().await {
                    if let Some(url) = msg["data"]["url"].as_str() {
                        let _ = LOBBY_URL.set(url.to_string());
                        println!("Lobby created: {}", url);
                    }
                }

                let _ = WEBRADAR.set(client);

                println!("WebRadar initialized");
            });
        });
    }

    fn web_radar_config<'a>(&self, config: &'a Config) -> &'a WebRadarConfig {
        &config.web_radar
    }
}
