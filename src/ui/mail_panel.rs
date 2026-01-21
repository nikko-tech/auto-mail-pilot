use eframe::egui;
use crate::models::{AppState, RecipientInfo};
use crate::api::GasClient;
use crate::utils::apply_variables;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.columns(3, |columns| {
        // Column 1: Selection (Templates & Recipients)
        columns[0].vertical(|ui| {
            ui.heading("1. 宛先とテンプレート");
            ui.separator();
            
            ui.label("宛先リスト:");
            egui::ScrollArea::vertical().id_salt("recipients_scroll").max_height(200.0).show(ui, |ui| {
                let mut sel_rec_idx = state.selected_recipient_index;
                for (i, rec) in state.recipients_master.iter().enumerate() {
                    let label = format!("{} ({})", rec.name, rec.company);
                    if ui.selectable_label(sel_rec_idx == Some(i), label).clicked() {
                        sel_rec_idx = Some(i);
                        
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
                state.selected_recipient_index = sel_rec_idx;
            });

            ui.separator();
            ui.label("テンプレート:");
            egui::ScrollArea::vertical().id_salt("templates_scroll").max_height(200.0).show(ui, |ui| {
                let mut selected_idx = state.selected_template_index;
                for (i, template) in state.templates.iter().enumerate() {
                    if ui.selectable_label(selected_idx == Some(i), &template.name).clicked() {
                        selected_idx = Some(i);
                        state.mail_draft.subject = template.subject.clone();
                        
                        // Apply template to ALL recipients who have data
                        for (r_idx, draft_rec) in state.mail_draft.recipients.iter_mut().enumerate() {
                            if !draft_rec.email.is_empty() {
                                let master_rec = state.recipients_master.iter().find(|m| m.email == draft_rec.email);
                                if let Some(m) = master_rec {
                                    draft_rec.body = apply_variables(template.body.clone(), m);
                                } else {
                                    draft_rec.body = template.body.clone();
                                }
                            } else if r_idx == state.active_recipient_index {
                                draft_rec.body = template.body.clone();
                            }
                        }
                        
                        // Save template selection to settings
                        if let Some(template) = state.templates.get(i) {
                            let client = GasClient::new(state.gas_url.clone());
                            let mut settings = std::collections::HashMap::new();
                            settings.insert("selected_template_id".to_string(), template.id.clone());
                            let _ = client.save_settings(&settings);
                        }
                    }
                }
                state.selected_template_index = selected_idx;
            });
        });

        // Column 2: Editor
        columns[1].vertical(|ui| {
            ui.heading("2. 内容の編集");
            ui.separator();
            
            ui.label("件名:");
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
