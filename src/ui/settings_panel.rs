use eframe::egui;
use crate::models::AppState;
use crate::api::GasClient;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Settings");
    ui.separator();

    ui.group(|ui| {
        ui.label("GAS Web App URL:");
        ui.text_edit_singleline(&mut state.gas_url);
        
        if ui.button("Test Connection").clicked() {
             let client = GasClient::new(state.gas_url.clone());
             match client.get_templates() {
                 Ok(_) => state.status_message = "Connection OK!".to_string(),
                 Err(e) => state.status_message = format!("Connection Failed: {}", e),
             }
        }
    });

    ui.add_space(20.0);
    ui.label("Note: Deploy the GAS script as a Web App and paste the URL here.");
}
