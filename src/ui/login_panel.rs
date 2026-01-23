use eframe::egui;
use crate::models::AppState;

/// ログイン画面を表示
pub fn show(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            // タイトル
            ui.label(egui::RichText::new("🔐 Auto Mail Pilot")
                .size(32.0)
                .strong());

            ui.add_space(8.0);
            ui.label(egui::RichText::new("ログインしてください")
                .size(16.0)
                .color(egui::Color32::from_rgb(180, 180, 180)));

            ui.add_space(40.0);

            // ログインフォーム
            egui::Frame::none()
                .fill(egui::Color32::from_gray(35))
                .inner_margin(32.0)
                .rounding(8.0)
                .show(ui, |ui| {
                    ui.set_min_width(300.0);

                    // ユーザー名
                    ui.label("ユーザー名");
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(50, 80, 120))
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 120, 170)))
                        .inner_margin(8.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.auth_username)
                                .hint_text("ユーザー名を入力...")
                                .text_color(egui::Color32::WHITE)
                                .frame(false)
                                .desired_width(f32::INFINITY));
                        });

                    ui.add_space(16.0);

                    // パスワード
                    ui.label("パスワード");
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(50, 80, 120))
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 120, 170)))
                        .inner_margin(8.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.auth_password)
                                .hint_text("パスワードを入力...")
                                .password(true)
                                .text_color(egui::Color32::WHITE)
                                .frame(false)
                                .desired_width(f32::INFINITY));
                        });

                    ui.add_space(24.0);

                    // エラーメッセージ
                    if let Some(ref error) = state.auth_error {
                        ui.label(egui::RichText::new(error)
                            .color(egui::Color32::from_rgb(255, 100, 100)));
                        ui.add_space(8.0);
                    }

                    // ログインボタン
                    let can_login = !state.auth_username.is_empty()
                        && !state.auth_password.is_empty();

                    let button = egui::Button::new(
                        egui::RichText::new("ログイン").size(16.0)
                    )
                    .min_size(egui::vec2(f32::INFINITY, 40.0))
                    .fill(if can_login {
                        egui::Color32::from_rgb(50, 120, 80)
                    } else {
                        egui::Color32::from_rgb(60, 60, 60)
                    });

                    if ui.add_enabled(can_login, button).clicked() {
                        // 認証チェック
                        if state.auth_username == state.expected_username
                            && state.auth_password == state.expected_password {
                            state.is_authenticated = true;
                            state.auth_error = None;
                            state.status_message = "ログイン成功！".to_string();
                        } else {
                            state.auth_error = Some("ユーザー名またはパスワードが正しくありません".to_string());
                        }
                    }

                    // Enterキーでもログイン可能に
                    if can_login && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if state.auth_username == state.expected_username
                            && state.auth_password == state.expected_password {
                            state.is_authenticated = true;
                            state.auth_error = None;
                            state.status_message = "ログイン成功！".to_string();
                        } else {
                            state.auth_error = Some("ユーザー名またはパスワードが正しくありません".to_string());
                        }
                    }
                });
        });
    });
}
