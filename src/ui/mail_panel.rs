use eframe::egui;
use crate::models::{AppState, PendingSendData, PendingRecipient};
use crate::api::GasClient;
use crate::utils::{apply_variables, validate_send_safety};
use crate::file_utils::{extract_company_name_from_path, extract_filename_parts, encode_file_to_base64, get_mime_type};

/// 宛先を選択し、ロック状態を設定する
/// force_unlock: trueの場合、既存のロックを解除して新しい宛先を設定
pub fn select_recipient(state: &mut AppState, index: usize, force_unlock: bool) {
    let active_idx = state.active_recipient_index;

    // ロックチェック
    if let Some(draft_rec) = state.mail_draft.recipients.get(active_idx) {
        if draft_rec.locked_recipient_id.is_some() && !force_unlock {
            // ロックされている場合は警告メッセージを表示
            state.status_message = "⚠️ 宛先はロックされています。変更するには「ロック解除」を押してください".to_string();
            return;
        }
    }

    state.selected_recipient_index = Some(index);
    if let Some(rec) = state.recipients_master.get(index) {
        if let Some(draft_rec) = state.mail_draft.recipients.get_mut(active_idx) {
            draft_rec.email = rec.email.clone();
            // 宛先をロック
            draft_rec.locked_recipient_id = Some(rec.id.clone());
            draft_rec.locked_company = Some(rec.company.clone());

            // Auto-apply linked template if exists
            let linked_template = state.linkings_master.iter()
                .find(|link| link.recipient_id == rec.id)
                .and_then(|link| state.templates.iter()
                    .position(|t| t.id == link.template_id));

            if let Some(template_idx) = linked_template {
                state.selected_template_index = Some(template_idx);
                if let Some(template) = state.templates.get(template_idx) {
                    state.mail_draft.subject = template.subject.clone();
                    draft_rec.body = apply_variables(template.body.clone(), rec);
                }
            } else if let Some(t_idx) = state.selected_template_index {
                if let Some(template) = state.templates.get(t_idx) {
                    draft_rec.body = apply_variables(template.body.clone(), rec);
                }
            }
        }
    }
}

/// 現在アクティブな宛先のロックを解除
pub fn unlock_recipient(state: &mut AppState) {
    let active_idx = state.active_recipient_index;
    if let Some(draft_rec) = state.mail_draft.recipients.get_mut(active_idx) {
        draft_rec.locked_recipient_id = None;
        draft_rec.locked_company = None;
        state.status_message = "🔓 宛先のロックを解除しました".to_string();
    }
}

fn apply_template(state: &mut AppState, template_idx: usize) {
    if let Some(template) = state.templates.get(template_idx) {
        state.mail_draft.subject = template.subject.clone();
        let active_idx = state.active_recipient_index;

        let recipient_data = state.selected_recipient_index
            .and_then(|r_idx| state.recipients_master.get(r_idx).cloned());

        if let Some(draft_rec) = state.mail_draft.recipients.get_mut(active_idx) {
            if let Some(ref rec) = recipient_data {
                draft_rec.body = apply_variables(template.body.clone(), rec);
            } else {
                draft_rec.body = template.body.clone();
            }
        }
        state.status_message = format!("テンプレート「{}」を適用しました", template.name);
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    // Handle dropped files
    let dropped_files = ui.input(|i| {
        i.raw.dropped_files.iter()
            .filter_map(|f| f.path.clone())
            .collect::<Vec<_>>()
    });

    for path in dropped_files {
        let path_str = path.to_string_lossy();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        if extension == "csv" {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let client = GasClient::new(state.gas_url.clone());
                for line in content.lines().skip(1) {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        let rec = crate::models::RecipientData {
                            id: (state.recipients_master.len() + 1).to_string(),
                            company: parts[0].trim().to_string(),
                            name: parts[1].trim().to_string(),
                            email: parts[2].trim().to_string(),
                        };
                        let _ = client.save_recipient(&rec);
                        state.recipients_master.push(rec);
                    }
                }
                state.status_message = format!("CSVから宛先をインポートしました: {}", path_str);
            }
        } else {
            if let (Ok(data), name) = (encode_file_to_base64(&path_str), path.file_name()) {
                let file_name = name.map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string());
                let mime_type = get_mime_type(&file_name);

                // ファイル名から会社名を抽出
                let linked_company = extract_company_name_from_path(&path_str);
                let active_idx = state.active_recipient_index;

                state.mail_draft.attachments.push(crate::models::Attachment {
                    file_path: path_str.to_string(),
                    file_name: file_name.clone(),
                    enabled: true,
                    data,
                    mime_type,
                    linked_company: linked_company.clone(),
                    linked_recipient_index: Some(active_idx),  // 現在アクティブな宛先に紐付け
                });

                if let Some(company) = linked_company {
                    let company_normalized = company.replace(" ", "").replace("　", "");

                    // Auto-select recipient from filename
                    if let Some(pos) = state.recipients_master.iter()
                        .position(|r| {
                            let company_norm = r.company.replace(" ", "").replace("　", "");
                            let name_norm = r.name.replace(" ", "").replace("　", "");
                            let combined = format!("{}{}", name_norm, company_norm);

                            company_norm.contains(&company_normalized)
                                || company_normalized.contains(&company_norm)
                                || name_norm.contains(&company_normalized)
                                || combined.contains(&company_normalized)
                        })
                    {
                        select_recipient(state, pos, true);  // force_unlock = true for auto-selection
                        let rec = &state.recipients_master[pos];
                        let display_name = if rec.company.is_empty() {
                            rec.name.clone()
                        } else {
                            format!("{} ({})", rec.name, rec.company)
                        };
                        state.status_message = format!("🔒 ファイル名から宛先を自動選択＆ロック: {}", display_name);

                        // 添付ファイルの紐付けを更新
                        if let Some(att) = state.mail_draft.attachments.last_mut() {
                            att.linked_recipient_index = Some(state.active_recipient_index);
                        }
                    }

                }

                // Auto-select template from filename (use all filename parts)
                let filename_parts = extract_filename_parts(&path_str);
                if let Some(template_pos) = state.templates.iter()
                    .position(|t| {
                        let template_name_norm = t.name.replace(" ", "").replace("　", "");
                        filename_parts.iter().any(|part| {
                            let part_norm = part.replace(" ", "").replace("　", "");
                            template_name_norm.contains(&part_norm)
                                || part_norm.contains(&template_name_norm)
                        })
                    })
                {
                    state.selected_template_index = Some(template_pos);
                    apply_template(state, template_pos);
                    let template_name = &state.templates[template_pos].name;
                    state.status_message = format!("ファイル名からテンプレートを自動選択: {}", template_name);
                }
            }
        }
    }

    // ========== TOP SECTION: Recipients & Templates (dropdowns) ==========
    ui.horizontal(|ui| {
        // --- Recipients dropdown ---
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(8.0)
            .outer_margin(2.0)
            .rounding(4.0)
            .show(ui, |ui| {
                ui.set_min_width(220.0);
                ui.vertical(|ui| {
                    // ロック状態を確認
                    let active_idx = state.active_recipient_index;
                    let is_locked = state.mail_draft.recipients
                        .get(active_idx)
                        .and_then(|r| r.locked_recipient_id.as_ref())
                        .is_some();
                    let locked_company = state.mail_draft.recipients
                        .get(active_idx)
                        .and_then(|r| r.locked_company.clone())
                        .unwrap_or_default();

                    ui.horizontal(|ui| {
                        if is_locked {
                            ui.strong("🔒 宛先");
                        } else {
                            ui.strong("👤 宛先");
                        }
                        ui.add_space(8.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(50, 80, 120))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 170)))
                            .inner_margin(4.0)
                            .rounding(3.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut state.recipient_search)
                                    .hint_text("🔍 検索...")
                                    .text_color(egui::Color32::WHITE)
                                    .frame(false)
                                    .desired_width(90.0));
                            });
                        if ui.small_button("➕").on_hover_text("新規宛先").clicked() {
                            let new_rec = crate::models::RecipientData {
                                id: (state.recipients_master.len() + 1).to_string(),
                                company: "新規会社".to_string(),
                                name: "氏名".to_string(),
                                email: "".to_string(),
                            };
                            state.recipients_master.push(new_rec);
                        }
                    });

                    // ロック状態の表示とロック解除ボタン
                    if is_locked {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("🔒 {}", locked_company))
                                .color(egui::Color32::from_rgb(255, 200, 100))
                                .small());
                            if ui.small_button("解除").on_hover_text("宛先ロックを解除").clicked() {
                                unlock_recipient(state);
                            }
                        });
                    }

                    ui.add_space(4.0);

                    let search_lower = state.recipient_search.to_lowercase();
                    let filtered_recipients: Vec<(usize, String)> = state.recipients_master.iter()
                        .enumerate()
                        .filter(|(_, r)| {
                            search_lower.is_empty()
                            || r.name.to_lowercase().contains(&search_lower)
                            || r.company.to_lowercase().contains(&search_lower)
                            || r.email.to_lowercase().contains(&search_lower)
                        })
                        .map(|(i, r)| {
                            let display = if r.company.is_empty() {
                                r.name.clone()
                            } else {
                                format!("{} - {}", r.company, r.name)
                            };
                            (i, display)
                        })
                        .collect();

                    egui::ScrollArea::vertical()
                        .id_salt("recipients_dropdown")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            let mut clicked_idx = None;
                            for (i, label) in &filtered_recipients {
                                let is_selected = state.selected_recipient_index == Some(*i);
                                let response = ui.selectable_label(is_selected, label);
                                if response.clicked() {
                                    clicked_idx = Some(*i);
                                }
                            }
                            if let Some(i) = clicked_idx {
                                select_recipient(state, i, false);  // force_unlock = false
                            }
                            if filtered_recipients.is_empty() {
                                ui.weak("宛先なし");
                            }
                        });
                });
            });

        ui.add_space(8.0);

        // --- Templates dropdown ---
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(8.0)
            .outer_margin(2.0)
            .rounding(4.0)
            .show(ui, |ui| {
                ui.set_min_width(220.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong("📝 テンプレート");
                        ui.add_space(8.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(50, 80, 120))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 170)))
                            .inner_margin(4.0)
                            .rounding(3.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut state.template_search)
                                    .hint_text("🔍 検索...")
                                    .text_color(egui::Color32::WHITE)
                                    .frame(false)
                                    .desired_width(70.0));
                            });
                        if ui.small_button("➕").on_hover_text("新規テンプレート").clicked() {
                            let new_temp = crate::models::Template {
                                id: (state.templates.len() + 1).to_string(),
                                name: "新しいテンプレート".to_string(),
                                subject: "".to_string(),
                                body: "".to_string(),
                            };
                            state.templates.push(new_temp);
                        }
                    });

                    ui.add_space(4.0);

                    let search_lower = state.template_search.to_lowercase();
                    let filtered_templates: Vec<(usize, String)> = state.templates.iter()
                        .enumerate()
                        .filter(|(_, t)| {
                            search_lower.is_empty()
                            || t.name.to_lowercase().contains(&search_lower)
                            || t.subject.to_lowercase().contains(&search_lower)
                        })
                        .map(|(i, t)| (i, t.name.clone()))
                        .collect();

                    egui::ScrollArea::vertical()
                        .id_salt("templates_dropdown")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            let mut apply_idx = None;
                            for (i, label) in &filtered_templates {
                                let is_selected = state.selected_template_index == Some(*i);
                                if ui.selectable_label(is_selected, label).clicked() {
                                    state.selected_template_index = Some(*i);
                                    apply_idx = Some(*i);
                                }
                            }
                            if let Some(i) = apply_idx {
                                apply_template(state, i);
                            }
                            if filtered_templates.is_empty() {
                                ui.weak("テンプレートなし");
                            }
                        });
                });
            });

        ui.add_space(8.0);

        // --- Signature selector ---
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(8.0)
            .outer_margin(2.0)
            .rounding(4.0)
            .show(ui, |ui| {
                ui.set_min_width(150.0);
                ui.vertical(|ui| {
                    ui.strong("✍ 署名");
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("signatures_dropdown")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            let mut sel_sig_idx = state.selected_signature_index;
                            for (i, sig) in state.signatures.iter().enumerate() {
                                if ui.selectable_label(sel_sig_idx == Some(i), &sig.name).clicked() {
                                    sel_sig_idx = Some(i);
                                    let client = GasClient::new(state.gas_url.clone());
                                    let mut settings = std::collections::HashMap::new();
                                    settings.insert("selected_signature_index".to_string(), i.to_string());
                                    let _ = client.save_settings(&settings);
                                }
                            }
                            state.selected_signature_index = sel_sig_idx;
                            if state.signatures.is_empty() {
                                ui.weak("署名なし");
                            }
                        });
                });
            });
    });

    ui.add_space(8.0);

    // ========== MIDDLE SECTION: Email Editor ==========
    egui::Frame::none()
        .fill(ui.visuals().window_fill)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            // Section header
            ui.horizontal(|ui| {
                ui.strong("📧 メール編集");
                ui.add_space(16.0);

                // Recipient Tabs
                for i in 0..3 {
                    let has_email = state.mail_draft.recipients.get(i)
                        .map(|r| !r.email.is_empty())
                        .unwrap_or(false);
                    let is_active = state.active_recipient_index == i;

                    let label = if has_email {
                        format!("宛先{} ✓", i + 1)
                    } else {
                        format!("宛先{}", i + 1)
                    };

                    let button = egui::Button::new(label)
                        .selected(is_active);

                    if ui.add(button).clicked() {
                        state.active_recipient_index = i;
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let active_idx = state.active_recipient_index;

            // Get company name for display
            let company_display = state.selected_recipient_index
                .and_then(|idx| state.recipients_master.get(idx))
                .map(|r| {
                    if r.company.is_empty() {
                        r.name.clone()
                    } else {
                        format!("{} / {}", r.company, r.name)
                    }
                })
                .unwrap_or_default();

            if let Some(recipient) = state.mail_draft.recipients.get_mut(active_idx) {
                // To field
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("To:").strong());
                    ui.add_space(24.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(50, 80, 120))
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 120, 170)))
                        .inner_margin(6.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut recipient.email)
                                .hint_text("メールアドレス")
                                .text_color(egui::Color32::WHITE)
                                .frame(false)
                                .desired_width(280.0));
                        });
                    if !company_display.is_empty() {
                        ui.label(egui::RichText::new(format!("← {}", company_display))
                            .color(egui::Color32::from_rgb(120, 180, 255)));
                    }
                });

                ui.add_space(8.0);

                // Subject field
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("件名:").strong());
                    ui.add_space(12.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(50, 80, 120))
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 120, 170)))
                        .inner_margin(6.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.mail_draft.subject)
                                .hint_text("件名を入力")
                                .text_color(egui::Color32::WHITE)
                                .frame(false)
                                .desired_width(f32::INFINITY));
                        });
                });

                ui.add_space(8.0);

                // Body field
                ui.label(egui::RichText::new("本文:").strong());
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(50, 80, 120))
                    .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 120, 170)))
                    .inner_margin(8.0)
                    .rounding(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("body_editor")
                            .max_height(180.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut recipient.body)
                                    .hint_text("本文を入力...")
                                    .text_color(egui::Color32::WHITE)
                                    .frame(false)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(10));
                            });
                    });

                // Signature preview
                if let Some(sig_idx) = state.selected_signature_index {
                    if let Some(sig) = state.signatures.get(sig_idx) {
                        ui.add_space(4.0);
                        ui.collapsing(format!("✍ 署名: {}", sig.name), |ui| {
                            egui::Frame::none()
                                .fill(ui.visuals().extreme_bg_color)
                                .inner_margin(8.0)
                                .rounding(4.0)
                                .show(ui, |ui| {
                                    ui.label(&sig.content);
                                });
                        });
                    }
                }
            }
        });

    ui.add_space(8.0);

    // ========== BOTTOM SECTION: Attachments & Send ==========
    ui.horizontal(|ui| {
        // Attachments section
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(8.0)
            .rounding(4.0)
            .show(ui, |ui| {
                ui.set_min_width(400.0);
                ui.horizontal(|ui| {
                    ui.strong("📎 添付ファイル");
                    ui.add_space(8.0);

                    if state.mail_draft.attachments.is_empty() {
                        ui.weak("ファイルをドロップして追加");
                    } else {
                        egui::ScrollArea::horizontal()
                            .id_salt("attachments_list")
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let mut to_remove = None;
                                    for (i, att) in state.mail_draft.attachments.iter_mut().enumerate() {
                                        egui::Frame::none()
                                            .fill(ui.visuals().widgets.inactive.bg_fill)
                                            .inner_margin(4.0)
                                            .rounding(3.0)
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.checkbox(&mut att.enabled, "");
                                                    ui.label(&att.file_name);
                                                    if ui.small_button("✕").on_hover_text("削除").clicked() {
                                                        to_remove = Some(i);
                                                    }
                                                });
                                            });
                                    }
                                    if let Some(i) = to_remove {
                                        state.mail_draft.attachments.remove(i);
                                    }
                                });
                            });
                    }
                });
            });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Send button - larger and more prominent
            let valid_count = state.mail_draft.recipients.iter()
                .filter(|r| !r.email.is_empty())
                .count();

            let send_label = if valid_count > 0 {
                format!("📧 送信 ({}件)", valid_count)
            } else {
                "📧 送信".to_string()
            };

            let button = egui::Button::new(egui::RichText::new(send_label).size(16.0))
                .min_size(egui::vec2(100.0, 36.0));

            if ui.add_enabled(valid_count > 0, button).clicked() {
                // 送信前検証を実行
                let errors = validate_send_safety(
                    &state.mail_draft.recipients,
                    &state.recipients_master,
                    &state.mail_draft.attachments,
                );

                if !errors.is_empty() {
                    // 検証エラーがある場合
                    state.validation_errors = errors;
                    state.status_message = "⚠️ 検証エラーがあります。確認してください".to_string();
                } else {
                    // 検証OK → 確認ダイアログを表示
                    // PendingSendDataを作成
                    let signature = state.selected_signature_index
                        .and_then(|idx| state.signatures.get(idx))
                        .map(|sig| format!("\n\n{}", sig.content))
                        .unwrap_or_default();

                    let pending_recipients: Vec<PendingRecipient> = state.mail_draft.recipients.iter()
                        .enumerate()
                        .filter(|(_, r)| !r.email.is_empty())
                        .map(|(idx, rec)| {
                            let recipient_data = rec.locked_recipient_id.as_ref()
                                .and_then(|id| state.recipients_master.iter().find(|r| &r.id == id));

                            let attachments: Vec<String> = state.mail_draft.attachments.iter()
                                .filter(|a| a.enabled && a.linked_recipient_index == Some(idx))
                                .map(|a| a.file_name.clone())
                                .collect();

                            PendingRecipient {
                                email: rec.email.clone(),
                                company: recipient_data.map(|r| r.company.clone()).unwrap_or_default(),
                                name: recipient_data.map(|r| r.name.clone()).unwrap_or_default(),
                                body: format!("{}{}", rec.body, signature),
                                attachments,
                            }
                        })
                        .collect();

                    state.pending_send_data = Some(PendingSendData {
                        recipients: pending_recipients,
                        subject: state.mail_draft.subject.clone(),
                    });

                    state.show_send_confirmation = true;
                    state.confirmation_company_input = String::new();
                    state.confirmation_checked = false;
                    state.validation_errors.clear();
                }
            }
        });
    });

    // 検証エラー表示
    if !state.validation_errors.is_empty() {
        ui.add_space(8.0);
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(80, 30, 30))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 80, 80)))
            .inner_margin(12.0)
            .rounding(6.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("⚠️ 送信前検証エラー").strong().color(egui::Color32::from_rgb(255, 150, 150)));
                ui.add_space(8.0);
                for error in &state.validation_errors {
                    ui.label(egui::RichText::new(error).color(egui::Color32::from_rgb(255, 200, 200)));
                }
                ui.add_space(8.0);
                if ui.button("確認しました").clicked() {
                    state.validation_errors.clear();
                }
            });
    }

    // 送信前確認ダイアログ
    if state.show_send_confirmation {
        show_send_confirmation_dialog(ui, state);
    }
}

/// 送信前確認ダイアログを表示
fn show_send_confirmation_dialog(ui: &mut egui::Ui, state: &mut AppState) {
    // pending_send_dataをクローンして借用問題を回避
    let pending_clone = state.pending_send_data.clone();

    let mut should_close = false;
    let mut should_send = false;

    egui::Window::new("⚠️ 送信前確認")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.set_min_width(450.0);

            if let Some(ref pending) = pending_clone {
                ui.label(egui::RichText::new("以下の内容で送信します。宛先が正しいことを確認してください。")
                    .color(egui::Color32::from_rgb(255, 200, 100)));

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // 各宛先の情報を表示
                for (i, recipient) in pending.recipients.iter().enumerate() {
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .inner_margin(8.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(format!("【宛先{}】", i + 1)).strong());
                            ui.horizontal(|ui| {
                                ui.label("会社名:");
                                ui.label(egui::RichText::new(&recipient.company)
                                    .color(egui::Color32::from_rgb(100, 200, 255)));
                            });
                            ui.horizontal(|ui| {
                                ui.label("氏名:");
                                ui.label(&recipient.name);
                            });
                            ui.horizontal(|ui| {
                                ui.label("メール:");
                                ui.label(&recipient.email);
                            });
                            if !recipient.attachments.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label("添付:");
                                    ui.label(recipient.attachments.join(", "));
                                });
                            }
                        });
                    ui.add_space(4.0);
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // 確認入力
                let first_company = pending.recipients.first()
                    .map(|r| r.company.clone())
                    .unwrap_or_default();

                ui.label(egui::RichText::new("確認のため、送信先の会社名を入力してください:")
                    .color(egui::Color32::from_rgb(255, 200, 100)));
                ui.add_space(4.0);

                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(50, 80, 120))
                    .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 120, 170)))
                    .inner_margin(6.0)
                    .rounding(4.0)
                    .show(ui, |ui| {
                        ui.add(egui::TextEdit::singleline(&mut state.confirmation_company_input)
                            .hint_text("会社名を入力...")
                            .text_color(egui::Color32::WHITE)
                            .frame(false)
                            .desired_width(f32::INFINITY));
                    });

                // 入力が一致しているかチェック
                let input_normalized = state.confirmation_company_input
                    .replace(" ", "").replace("　", "").to_lowercase();
                let expected_normalized = first_company
                    .replace(" ", "").replace("　", "").to_lowercase();
                let input_matches = !input_normalized.is_empty()
                    && (input_normalized == expected_normalized
                        || expected_normalized.contains(&input_normalized)
                        || input_normalized.contains(&expected_normalized));

                if input_matches {
                    ui.label(egui::RichText::new("✓ 一致しています")
                        .color(egui::Color32::from_rgb(100, 255, 100)));
                } else if !state.confirmation_company_input.is_empty() {
                    ui.label(egui::RichText::new("✗ 会社名が一致しません")
                        .color(egui::Color32::from_rgb(255, 100, 100)));
                }

                ui.add_space(8.0);

                // チェックボックス
                ui.checkbox(&mut state.confirmation_checked,
                    "宛先・添付ファイルが正しいことを確認しました");

                ui.add_space(12.0);

                // ボタン
                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        should_close = true;
                    }

                    ui.add_space(16.0);

                    let can_send = input_matches && state.confirmation_checked;

                    let send_button = egui::Button::new(
                        egui::RichText::new("📧 送信する").size(14.0)
                    ).fill(if can_send {
                        egui::Color32::from_rgb(50, 120, 50)
                    } else {
                        egui::Color32::from_rgb(80, 80, 80)
                    });

                    if ui.add_enabled(can_send, send_button).clicked() {
                        should_send = true;
                    }
                });
            }
        });

    // ダイアログを閉じる処理
    if should_close {
        state.show_send_confirmation = false;
        state.pending_send_data = None;
        state.confirmation_company_input.clear();
        state.confirmation_checked = false;
    }

    // 送信処理
    if should_send {
        if let Some(ref pending) = state.pending_send_data {
            let client = GasClient::new(state.gas_url.clone());

            let items: Vec<(String, String, String)> = pending.recipients.iter()
                .map(|rec| (
                    rec.email.clone(),
                    pending.subject.clone(),
                    rec.body.clone()
                ))
                .collect();

            let items_ref: Vec<(&str, &str, &str)> = items.iter()
                .map(|(to, sub, body)| (to.as_str(), sub.as_str(), body.as_str()))
                .collect();

            match client.send_batch_mail(items_ref, &state.mail_draft.attachments) {
                Ok(_) => state.status_message = "✅ すべて送信完了しました！".to_string(),
                Err(e) => state.status_message = format!("❌ 送信エラー: {}", e),
            }
        }

        // ダイアログを閉じる
        state.show_send_confirmation = false;
        state.pending_send_data = None;
        state.confirmation_company_input.clear();
        state.confirmation_checked = false;
    }
}
