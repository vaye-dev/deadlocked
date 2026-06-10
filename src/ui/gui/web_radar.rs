use crate::ui::{
    app::App,
    gui::helpers::{checkbox, text_edit},
};
use egui::Ui;

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

        ui.horizontal(|ui| {
            ui.add_enabled(
                false,
                egui::TextEdit::singleline(&mut self.config.web_radar.lobby_link.clone()),
            );
            if ui.button("\u{f0c5}").clicked() {
                ui.ctx().copy_text(self.config.web_radar.lobby_link.clone());
            }
            ui.label("Web Link");
        });
    }
}
