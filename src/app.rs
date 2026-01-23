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

        // Set up custom visual styling for better contrast
        let mut style = (*cc.egui_ctx.style()).clone();

        // Make text edit fields more visible with lighter blue tint
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 85, 125);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 170));
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(65, 100, 145);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 200));
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(75, 115, 165);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(120, 170, 220));

        // Make selection more visible with blue accent
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(60, 100, 150);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 200));

        // Extreme background (used for section panels)
        style.visuals.extreme_bg_color = egui::Color32::from_gray(25);

        // Window/panel backgrounds
        style.visuals.window_fill = egui::Color32::from_gray(30);
        style.visuals.panel_fill = egui::Color32::from_gray(28);

        cc.egui_ctx.set_style(style);

        let mut state = AppState::default();

        // Load settings from GAS on startup
        if !state.gas_url.is_empty() {
            let client = crate::api::GasClient::new(state.gas_url.clone());

            // Auto connection test on startup
            state.status_message = "GASに接続中...".to_string();
            state.is_loading = true;

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

            let mut errors: Vec<String> = Vec::new();

            // Fetch each data independently to avoid one failure blocking everything
            match client.get_templates() {
                Ok(templates) => state.templates = templates,
                Err(e) => errors.push(format!("テンプレート: {}", e)),
            }
            match client.get_recipients() {
                Ok(recipients) => state.recipients_master = recipients,
                Err(e) => errors.push(format!("宛先: {}", e)),
            }
            match client.get_signatures() {
                Ok(signatures) => state.signatures = signatures,
                Err(e) => errors.push(format!("署名: {}", e)),
            }
            match client.get_linkings() {
                Ok(linkings) => state.linkings_master = linkings,
                Err(e) => errors.push(format!("紐付け: {}", e)),
            }
            match client.get_history() {
                Ok(history) => state.history = history,
                Err(_) => {} // 履歴がない場合はエラーとしない
            }

            if errors.is_empty() {
                state.status_message = "起動完了".to_string();
            } else {
                state.status_message = format!("⚠ 一部データ取得失敗: {}", errors.join(", "));
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

        // Top tab bar (system tabs style)
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);

                let tab_button = |ui: &mut egui::Ui, current: &mut Tab, target: Tab, label: &str| {
                    let is_selected = *current == target;
                    let text = if is_selected {
                        egui::RichText::new(label).strong()
                    } else {
                        egui::RichText::new(label)
                    };
                    if ui.selectable_label(is_selected, text).clicked() {
                        *current = target;
                    }
                };

                tab_button(ui, &mut state.tab, Tab::Main, "✉ メール作成");
                ui.add_space(16.0);
                tab_button(ui, &mut state.tab, Tab::History, "📜 送信履歴");
                ui.add_space(16.0);
                tab_button(ui, &mut state.tab, Tab::Settings, "⚙ 設定");
            });
            ui.add_space(4.0);
        });

        // Status bar at bottom with color coding
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            let msg = &state.status_message;

            // Determine status type and color
            let (icon, color) = if msg.contains("エラー") || msg.contains("失敗") {
                ("❌", egui::Color32::from_rgb(220, 80, 80))
            } else if msg.contains("成功") || msg.contains("完了") {
                ("✅", egui::Color32::from_rgb(80, 180, 80))
            } else if msg.contains("中...") || msg.contains("接続中") {
                ("⏳", egui::Color32::from_rgb(180, 180, 80))
            } else if msg.contains("自動選択") || msg.contains("適用") {
                ("✨", egui::Color32::from_rgb(100, 150, 220))
            } else {
                ("ℹ", ui.visuals().text_color())
            };

            ui.horizontal(|ui| {
                ui.label(icon);
                ui.label(egui::RichText::new(msg).color(color));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match state.tab {
                Tab::Main => ui::mail_panel::show(ui, &mut state),
                Tab::History => ui::history_panel::show(ui, &mut state),
                Tab::Settings => ui::settings_panel::show(ui, &mut state),
            }
        });
    }
}
