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
            // CSV Import Logic
            if let Ok(content) = std::fs::read_to_string(&path) {
                let client = GasClient::new(state.gas_url.clone());
                for line in content.lines().skip(1) { // Skip header
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
            // Normal attachment logic
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

                // Auto-select recipient based on filename
                if let Some(company) = extract_company_name_from_path(&path_str) {
                    // Normalize for matching (remove spaces)
                    let company_normalized = company.replace(" ", "").replace("　", "");

                    if let Some(pos) = state.recipients_master.iter()
                        .position(|r| {
                            // Check both company and name fields for match
                            let company_norm = r.company.replace(" ", "").replace("　", "");
                            let name_norm = r.name.replace(" ", "").replace("　", "");
                            let combined = format!("{}{}", name_norm, company_norm);

                            // Check if company from filename matches any field
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
                }
            }
        }
    }

    ui.columns(3, |columns| {
        // Column 1: Selection (Templates & Recipients)
        columns[0].vertical(|ui| {
            ui.heading("1. 宛先とテンプレート");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("宛先リスト:");
                if ui.button("➕").on_hover_text("新規宛先を追加").clicked() {
                    let new_rec = crate::models::RecipientData {
                        id: (state.recipients_master.len() + 1).to_string(),
                        company: "新規会社".to_string(),
                        name: "氏名".to_string(),
                        email: "".to_string(),
                    };
                    state.recipients_master.push(new_rec);
                }
            });

            egui::ScrollArea::vertical().id_salt("recipients_scroll").max_height(200.0).show(ui, |ui| {
                let mut clicked_idx = None;
                let mut delete_idx = None;
                for (i, rec) in state.recipients_master.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let label = format!("{} ({})", rec.name, rec.company);
                        if ui.selectable_label(state.selected_recipient_index == Some(i), label).clicked() {
                            clicked_idx = Some(i);
                        }
                        if ui.button("🗑").clicked() {
                            delete_idx = Some(i);
                        }
                    });
                }
                if let Some(i) = clicked_idx {
                    select_recipient(state, i);
                }
                if let Some(i) = delete_idx {
                    state.recipients_master.remove(i);
                    state.selected_recipient_index = None;
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("テンプレート:");
                if ui.button("➕").on_hover_text("新規テンプレートを追加").clicked() {
                    let new_temp = crate::models::Template {
                        id: (state.templates.len() + 1).to_string(),
                        name: "新しいテンプレート".to_string(),
                        subject: "".to_string(),
                        body: "".to_string(),
                    };
                    state.templates.push(new_temp);
                }
            });

            egui::ScrollArea::vertical().id_salt("templates_scroll").max_height(200.0).show(ui, |ui| {
                let mut selected_idx = state.selected_template_index;
                let mut delete_idx = None;
                let mut apply_template_idx: Option<usize> = None;
                for (i, template) in state.templates.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(selected_idx == Some(i), &template.name).clicked() {
                            selected_idx = Some(i);
                            apply_template_idx = Some(i);
                        }
                        if ui.button("🗑").clicked() {
                            delete_idx = Some(i);
                        }
                    });
                }
                state.selected_template_index = selected_idx;

                // Apply template when selected (auto-apply on click)
                if let Some(t_idx) = apply_template_idx {
                    if let Some(template) = state.templates.get(t_idx) {
                        state.mail_draft.subject = template.subject.clone();
                        let active_idx = state.active_recipient_index;

                        // Get recipient data for variable substitution
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

                if let Some(i) = delete_idx {
                    let client = GasClient::new(state.gas_url.clone());
                    if let Some(t) = state.templates.get(i) {
                        let _ = client.delete_template(&t.name);
                    }
                    state.templates.remove(i);
                    state.selected_template_index = None;
                }
            });

            // Manual template apply button
            if ui.button("📝 テンプレート適用").on_hover_text("選択中のテンプレートを本文に適用").clicked() {
                if let Some(t_idx) = state.selected_template_index {
                    if let Some(template) = state.templates.get(t_idx).cloned() {
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
                } else {
                    state.status_message = "テンプレートを選択してください".to_string();
                }
            }
        });

        // Column 2: Editor
        columns[1].vertical(|ui| {
            ui.heading("2. 内容の編集");
            ui.separator();

            // --- Master Data Quick Editor ---
            if let Some(_idx) = state.selected_template_index {
                ui.group(|ui| {
                    ui.label("📝 テンプレート編集:");
                    if let Some(template) = state.templates.get_mut(_idx) {
                        ui.horizontal(|ui| {
                            ui.label("名:"); ui.text_edit_singleline(&mut template.name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("件:"); ui.text_edit_singleline(&mut template.subject);
                        });
                        ui.label("本文:");
                        ui.text_edit_multiline(&mut template.body);
                        if ui.button("💾 GASに保存").clicked() {
                            let client = GasClient::new(state.gas_url.clone());
                            match client.save_template(template) {
                                Ok(_) => state.status_message = "テンプレートを保存しました".to_string(),
                                Err(e) => state.status_message = format!("保存失敗: {}", e),
                            }
                        }
                    }
                });
            }

            if let Some(idx) = state.selected_recipient_index {
                ui.group(|ui| {
                    ui.label("👤 宛先情報編集:");
                    if let Some(rec) = state.recipients_master.get_mut(idx) {
                        ui.horizontal(|ui| {
                            ui.label("会社:"); ui.text_edit_singleline(&mut rec.company);
                        });
                        ui.horizontal(|ui| {
                            ui.label("氏名:"); ui.text_edit_singleline(&mut rec.name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("メール:"); ui.text_edit_singleline(&mut rec.email);
                        });
                        if ui.button("💾 GASに保存").clicked() {
                            let client = GasClient::new(state.gas_url.clone());
                            match client.save_recipient(rec) {
                                Ok(_) => state.status_message = "宛先情報を保存しました".to_string(),
                                Err(e) => state.status_message = format!("保存失敗: {}", e),
                            }
                        }
                    }
                });
            }
            ui.separator();
            
            ui.label("作成中のメール件名:");
            ui.text_edit_singleline(&mut state.mail_draft.subject);
            
            ui.add_space(8.0);
            ui.label("署名:");
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                let mut sel_sig_idx = state.selected_signature_index;
                for (i, sig) in state.signatures.iter().enumerate() {
                    if ui.selectable_label(sel_sig_idx == Some(i), &sig.name).clicked() {
                        sel_sig_idx = Some(i);
                        
                        // Save signature selection to settings
                        let client = GasClient::new(state.gas_url.clone());
                        let mut settings = std::collections::HashMap::new();
                        settings.insert("selected_signature_index".to_string(), i.to_string());
                        let _ = client.save_settings(&settings);
                    }
                }
                state.selected_signature_index = sel_sig_idx;
            });

            ui.add_space(8.0);
            
            // Recipient Tabs
            ui.horizontal(|ui| {
                for i in 0..3 {
                    let label = format!("宛先{}", i + 1);
                    if ui.selectable_label(state.active_recipient_index == i, label).clicked() {
                        state.active_recipient_index = i;
                    }
                }
            });
            
            let active_idx = state.active_recipient_index;
            if let Some(recipient) = state.mail_draft.recipients.get_mut(active_idx) {
                ui.group(|ui| {
                    ui.label(format!("宛先{}:", active_idx + 1));
                    ui.text_edit_singleline(&mut recipient.email);
                    ui.add_space(4.0);
                    ui.label("本文:");
                    ui.text_edit_multiline(&mut recipient.body);
                    
                    // Signature preview
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label("📝 署名プレビュー:");
                    if let Some(sig_idx) = state.selected_signature_index {
                        if let Some(sig) = state.signatures.get(sig_idx) {
                            ui.group(|ui| {
                                ui.label(&sig.content);
                            });
                        }
                    } else {
                        ui.label("（署名が選択されていません）");
                    }
                });
            }
        });

        // Column 3: Send
        columns[2].vertical(|ui| {
            ui.heading("3. 送信");
            ui.separator();
            
            let mut valid_recipients = Vec::new();
            for (i, rec) in state.mail_draft.recipients.iter().enumerate() {
                if !rec.email.is_empty() {
                    valid_recipients.push((i, rec));
                }
            }

            if valid_recipients.is_empty() {
                ui.label("宛先が未選択です");
            } else {
                for (i, rec) in &valid_recipients {
                    ui.group(|ui| {
                        ui.label(format!("宛先{}:", i + 1));
                        ui.strong(&rec.email);
                    });
                }
            }
            
            ui.add_space(10.0);
            ui.label("📎 添付ファイル:");
            egui::ScrollArea::vertical().id_salt("attachments_scroll").max_height(100.0).show(ui, |ui| {
                let mut to_remove = None;
                for (i, att) in state.mail_draft.attachments.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut att.enabled, "");
                        ui.label(&att.file_name);
                        if ui.button("🗑").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }
                if let Some(i) = to_remove {
                    state.mail_draft.attachments.remove(i);
                }
            });

            if state.mail_draft.attachments.is_empty() {
                ui.weak("ファイルをドロップして追加");
            }
            
            ui.add_space(10.0);
            if ui.button("📧 一括送信する").clicked() {
                if !valid_recipients.is_empty() {
                     let client = GasClient::new(state.gas_url.clone());
                     state.status_message = "送信中...".to_string();
                     let signature = state.selected_signature_index
                        .and_then(|idx| state.signatures.get(idx))
                        .map(|sig| format!("\n\n{}", sig.content))
                        .unwrap_or_default();

                     let items: Vec<(&str, &str, String)> = valid_recipients.iter()
                        .map(|(_, rec)| (
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
            }
        });
    });
}
