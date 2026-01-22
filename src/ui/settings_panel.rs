use eframe::egui;
use crate::models::AppState;
use crate::api::GasClient;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("設定");
    ui.separator();

    ui.group(|ui| {
        ui.label("GAS ウェブアプリ URL:");
        ui.text_edit_singleline(&mut state.gas_url);
        
        if ui.button("接続テスト").clicked() {
            let client = GasClient::new(state.gas_url.clone());
            match client.get_templates() {
                Ok(_) => state.status_message = "✅ 接続成功！".to_string(),
                Err(e) => state.status_message = format!("❌ {}", e),
            }
        }
    });

    ui.add_space(20.0);
    ui.label("注意: URLは自動的に保存・固定されていますが、変更が必要な場合はこちらで編集可能です。");
}
