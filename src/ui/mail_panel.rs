use eframe::egui;
use crate::models::AppState;
use crate::api::GasClient;
use crate::utils::apply_variables;
use crate::file_utils::{extract_company_name_from_path, encode_file_to_base64, get_mime_type};

pub fn select_recipient(state: &mut AppState, index: usize) {
    state.selected_recipient_index = Some(index);
    if let Some(rec) = state.recipients_master.get(index) {
        let active_idx = state.active_recipient_index;
        if let Some(draft_rec) = state.mail_draft.recipients.get_mut(active_idx) {
            draft_rec.email = rec.email.clone();

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

                state.mail_draft.attachments.push(crate::models::Attachment {
                    file_path: path_str.to_string(),
                    file_name,
                    enabled: true,
                    data,
                    mime_type,
                });

                if let Some(company) = extract_company_name_from_path(&path_str) {
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
                        select_recipient(state, pos);
                        let rec = &state.recipients_master[pos];
                        let display_name = if rec.company.is_empty() {
                            rec.name.clone()
                        } else {
                            format!("{} ({})", rec.name, rec.company)
                        };
                        state.status_message = format!("ファイル名から宛先を自動選択: {}", display_name);
                    }

                    // Auto-select template from filename
                    if let Some(template_pos) = state.templates.iter()
                        .position(|t| {
                            let template_name_norm = t.name.replace(" ", "").replace("　", "");
                            template_name_norm.contains(&company_normalized)
                                || company_normalized.contains(&template_name_norm)
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
    }

    // ========== TOP SECTION: Recipients & Templates (dropdowns) ==========
    ui.group(|ui| {
        ui.horizontal(|ui| {
            // --- Recipients dropdown ---
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("宛先:");
                    ui.text_edit_singleline(&mut state.recipient_search);
                    if ui.button("➕").on_hover_text("新規宛先").clicked() {
                        let new_rec = crate::models::RecipientData {
                            id: (state.recipients_master.len() + 1).to_string(),
                            company: "新規会社".to_string(),
                            name: "氏名".to_string(),
                            email: "".to_string(),
                        };
                        state.recipients_master.push(new_rec);
                    }
                });

                let search_lower = state.recipient_search.to_lowercase();
                let filtered_recipients: Vec<(usize, String)> = state.recipients_master.iter()
                    .enumerate()
                    .filter(|(_, r)| {
                        search_lower.is_empty()
                        || r.name.to_lowercase().contains(&search_lower)
                        || r.company.to_lowercase().contains(&search_lower)
                        || r.email.to_lowercase().contains(&search_lower)
                    })
                    .map(|(i, r)| (i, format!("{} ({})", r.name, r.company)))
                    .collect();

                egui::ScrollArea::vertical().id_salt("recipients_dropdown").max_height(120.0).show(ui, |ui| {
                    let mut clicked_idx = None;
                    for (i, label) in &filtered_recipients {
                        if ui.selectable_label(state.selected_recipient_index == Some(*i), label).clicked() {
                            clicked_idx = Some(*i);
                        }
                    }
                    if let Some(i) = clicked_idx {
                        select_recipient(state, i);
                    }
                });
            });

            ui.separator();

            // --- Templates dropdown ---
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("テンプレート:");
                    ui.text_edit_singleline(&mut state.template_search);
                    if ui.button("➕").on_hover_text("新規テンプレート").clicked() {
                        let new_temp = crate::models::Template {
                            id: (state.templates.len() + 1).to_string(),
                            name: "新しいテンプレート".to_string(),
                            subject: "".to_string(),
                            body: "".to_string(),
                        };
                        state.templates.push(new_temp);
                    }
                });

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

                egui::ScrollArea::vertical().id_salt("templates_dropdown").max_height(120.0).show(ui, |ui| {
                    let mut apply_idx = None;
                    for (i, label) in &filtered_templates {
                        if ui.selectable_label(state.selected_template_index == Some(*i), label).clicked() {
                            state.selected_template_index = Some(*i);
                            apply_idx = Some(*i);
                        }
                    }
                    if let Some(i) = apply_idx {
                        apply_template(state, i);
                    }
                });
            });

            ui.separator();

            // --- Signature selector ---
            ui.vertical(|ui| {
                ui.label("署名:");
                egui::ScrollArea::vertical().id_salt("signatures_dropdown").max_height(120.0).show(ui, |ui| {
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
                });
            });
        });
    });

    ui.separator();

    // ========== MIDDLE SECTION: Email Editor ==========
    ui.group(|ui| {
        // Recipient Tabs
        ui.horizontal(|ui| {
            for i in 0..3 {
                let has_email = state.mail_draft.recipients.get(i)
                    .map(|r| !r.email.is_empty())
                    .unwrap_or(false);
                let label = if has_email {
                    format!("宛先{} ●", i + 1)
                } else {
                    format!("宛先{}", i + 1)
                };
                if ui.selectable_label(state.active_recipient_index == i, label).clicked() {
                    state.active_recipient_index = i;
                }
            }
        });

        ui.separator();

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
            ui.horizontal(|ui| {
                ui.label("宛先メール:");
                ui.add(egui::TextEdit::singleline(&mut recipient.email).desired_width(300.0));
                if !company_display.is_empty() {
                    ui.label(format!("【{}】", company_display));
                }
            });

            ui.horizontal(|ui| {
                ui.label("件名:");
                ui.add(egui::TextEdit::singleline(&mut state.mail_draft.subject).desired_width(400.0));
            });

            ui.add_space(4.0);
            ui.label("本文:");
            egui::ScrollArea::vertical().id_salt("body_editor").max_height(250.0).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut recipient.body)
                    .desired_width(f32::INFINITY)
                    .desired_rows(12));
            });

            // Signature preview
            if let Some(sig_idx) = state.selected_signature_index {
                if let Some(sig) = state.signatures.get(sig_idx) {
                    ui.separator();
                    ui.collapsing("署名プレビュー", |ui| {
                        ui.label(&sig.content);
                    });
                }
            }
        }
    });

    ui.separator();

    // ========== BOTTOM SECTION: Attachments & Send ==========
    ui.horizontal(|ui| {
        // Attachments
        ui.group(|ui| {
            ui.label("📎 添付ファイル:");
            egui::ScrollArea::horizontal().id_salt("attachments_list").show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut to_remove = None;
                    for (i, att) in state.mail_draft.attachments.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut att.enabled, "");
                                ui.label(&att.file_name);
                                if ui.small_button("✕").clicked() {
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
            if state.mail_draft.attachments.is_empty() {
                ui.weak("ファイルをドロップして追加");
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Send button
            let valid_count = state.mail_draft.recipients.iter()
                .filter(|r| !r.email.is_empty())
                .count();

            let send_label = if valid_count > 0 {
                format!("📧 送信 ({}件)", valid_count)
            } else {
                "📧 送信".to_string()
            };

            if ui.add_enabled(valid_count > 0, egui::Button::new(send_label)).clicked() {
                let client = GasClient::new(state.gas_url.clone());
                state.status_message = "送信中...".to_string();

                let signature = state.selected_signature_index
                    .and_then(|idx| state.signatures.get(idx))
                    .map(|sig| format!("\n\n{}", sig.content))
                    .unwrap_or_default();

                let valid_recipients: Vec<_> = state.mail_draft.recipients.iter()
                    .filter(|r| !r.email.is_empty())
                    .collect();

                let items: Vec<(&str, &str, String)> = valid_recipients.iter()
                    .map(|rec| (
                        rec.email.as_str(),
                        state.mail_draft.subject.as_str(),
                        format!("{}{}", rec.body, signature)
                    ))
                    .collect();

                let items_ref: Vec<(&str, &str, &str)> = items.iter()
                    .map(|(to, sub, body)| (*to, *sub, body.as_str()))
                    .collect();

                match client.send_batch_mail(items_ref, &state.mail_draft.attachments) {
                    Ok(_) => state.status_message = "すべて送信完了しました！".to_string(),
                    Err(e) => state.status_message = format!("送信エラー: {}", e),
                }
            }
        });
    });
}
