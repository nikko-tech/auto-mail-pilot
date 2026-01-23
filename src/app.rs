use eframe::egui;
use crate::models::{AppState, Tab, StartupPhase};
use crate::ui;
use std::sync::{Arc, Mutex};
use std::thread;

/// セッションファイルのパス（TEMPディレクトリに保存、PC再起動で消える）
fn get_session_file_path() -> std::path::PathBuf {
    std::env::temp_dir().join("auto_mail_pilot_session.txt")
}

/// セッションを保存（ログイン成功時）
fn save_session() {
    let session_path = get_session_file_path();
    let _ = std::fs::write(&session_path, "authenticated");
}

/// セッションを削除（ログアウト時）
fn clear_session() {
    let session_path = get_session_file_path();
    let _ = std::fs::remove_file(&session_path);
}

/// セッションが有効かチェック
fn check_session() -> bool {
    let session_path = get_session_file_path();
    session_path.exists()
}

pub struct MailApp {
    state: Arc<Mutex<AppState>>,
    loading_started: bool,
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

        // セッションが有効なら自動ログイン
        if check_session() {
            state.is_authenticated = true;
        }

        Self {
            state: Arc::new(Mutex::new(state)),
            loading_started: false,
        }
    }

    /// バックグラウンドでデータをロード
    fn start_loading(&mut self, ctx: egui::Context) {
        if self.loading_started {
            return;
        }
        self.loading_started = true;

        let state_clone = Arc::clone(&self.state);

        thread::spawn(move || {
            let gas_url = {
                let state = state_clone.lock().unwrap();
                state.gas_url.clone()
            };

            if gas_url.is_empty() {
                let mut state = state_clone.lock().unwrap();
                state.startup_phase = StartupPhase::Ready;
                ctx.request_repaint();
                return;
            }

            let client = crate::api::GasClient::new(gas_url);

            // ロード進捗を更新するヘルパー
            let update_message = |msg: &str| {
                if let Ok(mut state) = state_clone.lock() {
                    state.loading_message = msg.to_string();
                }
                ctx.request_repaint();
            };

            let mut errors: Vec<String> = Vec::new();

            // 設定を取得
            update_message("設定を読み込み中...");
            if let Ok(settings) = client.get_settings() {
                if let Ok(mut state) = state_clone.lock() {
                    if let Some(selected_signature_idx) = settings.get("selected_signature_index") {
                        if let Ok(idx) = selected_signature_idx.parse::<usize>() {
                            state.selected_signature_index = Some(idx);
                        }
                    }
                }
            }

            // テンプレート取得
            update_message("テンプレートを取得中...");
            match client.get_templates() {
                Ok(templates) => {
                    if let Ok(mut state) = state_clone.lock() {
                        state.templates = templates;
                    }
                }
                Err(e) => errors.push(format!("テンプレート: {}", e)),
            }

            // 宛先取得
            update_message("宛先マスターを取得中...");
            match client.get_recipients() {
                Ok(recipients) => {
                    if let Ok(mut state) = state_clone.lock() {
                        state.recipients_master = recipients;
                    }
                }
                Err(e) => errors.push(format!("宛先: {}", e)),
            }

            // 署名取得
            update_message("署名を取得中...");
            match client.get_signatures() {
                Ok(signatures) => {
                    if let Ok(mut state) = state_clone.lock() {
                        state.signatures = signatures.clone();
                        // デフォルト署名を選択
                        if state.selected_signature_index.is_none() && !signatures.is_empty() {
                            state.selected_signature_index = Some(0);
                        }
                    }
                }
                Err(e) => errors.push(format!("署名: {}", e)),
            }

            // 紐付けデータ取得
            update_message("紐付けデータを取得中...");
            match client.get_linkings() {
                Ok(linkings) => {
                    if let Ok(mut state) = state_clone.lock() {
                        state.linkings_master = linkings;
                    }
                }
                Err(e) => errors.push(format!("紐付け: {}", e)),
            }

            // 履歴取得
            update_message("送信履歴を取得中...");
            if let Ok(history) = client.get_history() {
                if let Ok(mut state) = state_clone.lock() {
                    state.history = history;
                }
            }

            // 完了
            if let Ok(mut state) = state_clone.lock() {
                if errors.is_empty() {
                    state.status_message = "起動完了".to_string();
                } else {
                    state.status_message = format!("⚠ 一部データ取得失敗: {}", errors.join(", "));
                }
                state.startup_phase = StartupPhase::Ready;
                state.is_loading = false;
            }
            ctx.request_repaint();
        });
    }
}

impl eframe::App for MailApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let startup_phase = {
            let state = self.state.lock().unwrap();
            state.startup_phase.clone()
        };

        // スプラッシュ画面またはローディング画面
        if startup_phase == StartupPhase::Splash || startup_phase == StartupPhase::Loading {
            self.show_splash_screen(ctx);

            // スプラッシュ表示後、ロードを開始
            if startup_phase == StartupPhase::Splash {
                {
                    let mut state = self.state.lock().unwrap();
                    state.startup_phase = StartupPhase::Loading;
                }
                self.start_loading(ctx.clone());
            }
            return;
        }

        let mut state = self.state.lock().unwrap();

        // 認証されていない場合はログイン画面を表示
        if !state.is_authenticated {
            ui::login_panel::show(ctx, &mut state);

            // ログイン成功時にセッションを保存
            if state.is_authenticated {
                save_session();
            }
            return;
        }

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

                // ログアウトボタン（右寄せ）
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🚪 ログアウト").clicked() {
                        state.is_authenticated = false;
                        state.auth_username.clear();
                        state.auth_password.clear();
                        state.auth_error = None;
                        clear_session();  // セッションを削除
                    }
                });
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

impl MailApp {
    /// スプラッシュスクリーンを表示
    fn show_splash_screen(&self, ctx: &egui::Context) {
        let loading_message = {
            let state = self.state.lock().unwrap();
            state.loading_message.clone()
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);

                // アプリアイコン / ロゴ
                ui.label(egui::RichText::new("✉")
                    .size(80.0)
                    .color(egui::Color32::from_rgb(100, 180, 255)));

                ui.add_space(16.0);

                // アプリ名
                ui.label(egui::RichText::new("Auto Mail Pilot")
                    .size(32.0)
                    .strong()
                    .color(egui::Color32::WHITE));

                ui.add_space(8.0);

                // サブタイトル
                ui.label(egui::RichText::new("メール送信自動化ツール")
                    .size(14.0)
                    .color(egui::Color32::from_rgb(150, 150, 150)));

                ui.add_space(40.0);

                // ローディングスピナー（アニメーション）
                let time = ui.input(|i| i.time);
                let spinner_chars = ["◐", "◓", "◑", "◒"];
                let spinner_idx = ((time * 8.0) as usize) % spinner_chars.len();
                ui.label(egui::RichText::new(spinner_chars[spinner_idx])
                    .size(24.0)
                    .color(egui::Color32::from_rgb(100, 180, 255)));

                ui.add_space(12.0);

                // ローディングメッセージ
                ui.label(egui::RichText::new(&loading_message)
                    .size(14.0)
                    .color(egui::Color32::from_rgb(180, 180, 180)));

                // 継続的に再描画してアニメーションを動かす
                ctx.request_repaint();
            });
        });
    }
}
