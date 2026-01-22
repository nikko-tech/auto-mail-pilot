use eframe::egui;
use crate::models::{AppState, Tab};
use crate::ui;
use std::sync::{Arc, Mutex};

pub struct MailApp {
    state: Arc<Mutex<AppState>>,
}

impl MailApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up Japanese font
        let mut fonts = egui::FontDefinitions::default();
        
        // Font paths to try: Windows and macOS system fonts
        let font_paths = [
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "C:\\Windows\\Fonts\\msgothic.ttc",
            "C:\\Windows\\Fonts\\msmincho.ttc",
        ];

        let mut font_loaded = false;
        for path in font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "jp_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                
                // Put it at the top of the priority list
                fonts.families.get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "jp_font".to_owned());
                fonts.families.get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "jp_font".to_owned());
                
                font_loaded = true;
                break;
            }
        }
        
        if !font_loaded {
            // Fallback: If no system font found, the user might see tofu, 
            // but we'll at least use the default egui fonts.
        }
        
        cc.egui_ctx.set_fonts(fonts);

        let mut state = AppState::default();
        
        // Load settings from GAS on startup
        if !state.gas_url.is_empty() {
            let client = crate::api::GasClient::new(state.gas_url.clone());

            // Auto connection test on startup
            state.status_message = "GASに接続中...".to_string();
            state.is_loading = true;

            let mut connection_ok = true;

            // Load saved settings
            if let Ok(settings) = client.get_settings() {
                if let Some(selected_signature_idx) = settings.get("selected_signature_index") {
                    if let Ok(idx) = selected_signature_idx.parse::<usize>() {
                        state.selected_signature_index = Some(idx);
                    }
                }
            }

            // Fetch each data independently to avoid one failure blocking everything
            state.templates = client.get_templates().unwrap_or_else(|e| {
                state.status_message = format!("エラー: テンプレート取得失敗 - {}", e);
                connection_ok = false;
                Vec::new()
            });
            state.recipients_master = client.get_recipients().unwrap_or_else(|e| {
                state.status_message = format!("エラー: 宛先取得失敗 - {}", e);
                connection_ok = false;
                Vec::new()
            });
            state.signatures = client.get_signatures().unwrap_or_else(|e| {
                state.status_message = format!("エラー: 署名取得失敗 - {}", e);
                connection_ok = false;
                Vec::new()
            });
            state.linkings_master = client.get_linkings().unwrap_or_else(|e| {
                state.status_message = format!("エラー: 紐付け取得失敗 - {}", e);
                connection_ok = false;
                Vec::new()
            });
            state.history = client.get_history().unwrap_or_else(|e| {
                state.status_message = format!("警告: 履歴なしまたは取得失敗 - {}", e);
                Vec::new()
            });

            if connection_ok {
                state.status_message = "接続成功！".to_string();
            }

            // Select default signature if not already set
            if state.selected_signature_index.is_none() && !state.signatures.is_empty() {
                state.selected_signature_index = Some(0);
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
                ui.selectable_value(&mut state.tab, Tab::History, "📜 送信履歴");
                ui.selectable_value(&mut state.tab, Tab::Settings, "⚙ 設定");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match state.tab {
                Tab::Main => ui::mail_panel::show(ui, &mut state),
                Tab::History => ui::history_panel::show(ui, &mut state),
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
