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

        Self {
            state: Arc::new(Mutex::new(AppState::default())),
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
