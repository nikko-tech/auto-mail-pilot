use eframe::egui;
use crate::models::AppState;
use crate::api::GasClient;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("送信履歴 (直近50件)");
    ui.separator();

    if ui.button("🔄 履歴を更新").clicked() {
        let client = GasClient::new(state.gas_url.clone());
        state.is_loading = true;
        state.status_message = "履歴を取得中...".to_string();
        
        match client.get_history() {
            Ok(logs) => {
                state.history = logs;
                state.status_message = "履歴を更新しました".to_string();
            }
            Err(e) => state.status_message = format!("履歴取得エラー: {}", e),
        }
        state.is_loading = false;
    }

    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        if state.history.is_empty() {
            ui.label("履歴がありません");
        } else {
            egui::Grid::new("history_grid")
                .num_columns(5)
                .spacing([10.0, 10.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("日時");
                    ui.label("宛先");
                    ui.label("件名");
                    ui.label("ステータス");
                    ui.label("操作");
                    ui.end_row();

                    for item in &state.history {
                        ui.label(&item.date);
                        ui.label(&item.to);
                        ui.label(&item.subject);
                        
                        if item.status.starts_with("Error") {
                            ui.colored_label(egui::Color32::RED, &item.status);
                        } else {
                            ui.colored_label(egui::Color32::GREEN, &item.status);
                        }

                        if ui.button("👁").on_hover_text("詳細を表示").clicked() {
                            // TODO: Show modal or expanded view of body
                        }
                        ui.end_row();
                    }
                });
        }
    });
}
