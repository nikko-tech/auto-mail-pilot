use eframe::egui;
use crate::models::{AppState, RecipientInfo};
use crate::api::GasClient;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.columns(3, |columns| {
        // Column 1: Recipient Selection
        columns[0].vertical(|ui| {
            ui.heading("1. Select Recipient");
            ui.separator();
            
            // This would mock a recipient list or fetch from Google Sheets
            if ui.button("Load Recipients (Mock)").clicked() {
                 let client = GasClient::new(state.gas_url.clone());
                 match client.get_templates() {
                     Ok(templates) => {
                         state.templates = templates;
                         state.status_message = "Templates loaded".to_string();
                     }
                     Err(e) => state.status_message = format!("Error: {}", e),
                 }
            }
            
            ui.separator();
            ui.label("Templates:");
            let mut selected_idx = state.selected_template_index;
            for (i, template) in state.templates.iter().enumerate() {
                if ui.selectable_label(selected_idx == Some(i), &template.name).clicked() {
                    selected_idx = Some(i);
                    // Autofill draft
                    state.mail_draft.subject = template.subject.clone();
                    state.mail_draft.recipients = vec![RecipientInfo {
                        email: "example@example.com".to_string(), // Placeholder
                        body: template.body.clone(),
                    }];
                }
            }
            state.selected_template_index = selected_idx;
        });

        // Column 2: Editor
        columns[1].vertical(|ui| {
            ui.heading("2. Edit Content");
            ui.separator();
            
            ui.label("Subject:");
            ui.text_edit_singleline(&mut state.mail_draft.subject);
            
            ui.label("Body:");
            if let Some(recipient) = state.mail_draft.recipients.get_mut(0) {
                 ui.text_edit_multiline(&mut recipient.body);
            } else {
                ui.label("Select a template to start editing.");
            }
        });

        // Column 3: Send
        columns[2].vertical(|ui| {
            ui.heading("3. Send");
            ui.separator();
            
            if let Some(recipient) = state.mail_draft.recipients.first() {
                ui.label(format!("To: {}", recipient.email));
            }
            
            if ui.button("SEND EMAIL").clicked() {
                if let Some(recipient) = state.mail_draft.recipients.first() {
                     let client = GasClient::new(state.gas_url.clone());
                     match client.send_mail(&recipient.email, &state.mail_draft.subject, &recipient.body) {
                         Ok(_) => state.status_message = "Email Sent Successfully!".to_string(),
                         Err(e) => state.status_message = format!("Send Error: {}", e),
                     }
                }
            }
        });
    });
}
