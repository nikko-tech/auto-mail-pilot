use eframe::egui;
use crate::models::{AppState, Tab};
use crate::api::GasClient;
use crate::ui;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct MailApp {
    state: Arc<Mutex<AppState>>,
}

impl MailApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up Japanese font
        let mut fonts = egui::FontDefinitions::default();
        
        // Load MS Gothic from Windows system fonts if available
        let font_path = "C:\\Windows\\Fonts\\msgothic.ttc";
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "jp_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            
            // Put it at the top of the priority list for proportional and monospace
            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "jp_font".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "jp_font".to_owned());
        }
        
        cc.egui_ctx.set_fonts(fonts);

        let mut state = AppState::default();
        
        // Load settings from GAS on startup
        if !state.gas_url.is_empty() {
            let client = crate::api::GasClient::new(state.gas_url.clone());
            
            // Load saved settings
            if let Ok(settings) = client.get_settings() {
                if let Some(selected_signature_idx) = settings.get("selected_signature_index") {
                    if let Ok(idx) = selected_signature_idx.parse::<usize>() {
                        state.selected_signature_index = Some(idx);
                    }
                }
            }
            
            // Auto-fetch master data on startup
            state.status_message = "マスターデータを取得中...".to_string();
            state.is_loading = true;
            
            let t_result = client.get_templates();
            let r_result = client.get_recipients();
            let s_result = client.get_signatures();
            let l_result = client.get_linkings();
            
            match (t_result, r_result, s_result, l_result) {
                (Ok(templates), Ok(recipients), Ok(signatures), Ok(linkings)) => {
                    state.templates = templates;
                    state.recipients_master = recipients;
                    state.signatures = signatures;
                    state.linkings_master = linkings;
                    state.status_message = "起動完了 - マスターデータ取得成功".to_string();
                    
                    // Select default signature if not already set
                    if state.selected_signature_index.is_none() && !state.signatures.is_empty() {
                        state.selected_signature_index = Some(0);
                    }
                }
                (Err(e), _, _, _) => state.status_message = format!("起動時エラー: テンプレート取得失敗 - {}", e),
                (_, Err(e), _, _) => state.status_message = format!("起動時エラー: 宛先リスト取得失敗 - {}", e),
                (_, _, Err(e), _) => state.status_message = format!("起動時エラー: 署名取得失敗 - {}", e),
                (_, _, _, Err(e)) => state.status_message = format!("起動時エラー: 紐付けマスター取得失敗 - {}", e),
            }
            
            state.is_loading = false;
        }

        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

impl eframe::App for MailApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();

        egui::SidePanel::left("nav_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("メニュー");
                ui.separator();
                ui.selectable_value(&mut state.tab, Tab::Main, "✉ メール作成");
                ui.selectable_value(&mut state.tab, Tab::Settings, "⚙ 設定");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match state.tab {
                Tab::Main => ui::mail_panel::show(ui, &mut state),
                Tab::Settings => ui::settings_panel::show(ui, &mut state),
            }
        });
        
        // Show status message at bottom
        if !state.status_message.is_empty() {
             egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.label(&state.status_message);
            });
        }
    }
}
