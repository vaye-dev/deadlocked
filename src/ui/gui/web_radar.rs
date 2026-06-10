use crate::ui::{
    app::App,
    gui::helpers::{checkbox, readonly_text, text_edit},
};
use egui::{DragValue, Ui};

impl App {
    pub fn web_radar_settings(&mut self, ui: &mut Ui) {
        if checkbox(ui, "Enabled", &mut self.config.web_radar.enabled) {
            self.send_config();
        }

        if text_edit(
            ui,
            "Web Endpoint",
            &mut self.config.web_radar.web_radar_endpoint,
        ) {
            self.send_config();
        }

        if text_edit(ui, "Web API Key", &mut self.config.web_radar.web_radar_api) {
            self.send_config();
        }

        readonly_text(ui, "Web Link", &mut self.config.web_radar.lobby_link);
    }
}
