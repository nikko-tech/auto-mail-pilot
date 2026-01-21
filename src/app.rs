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
                ui.heading("Menu");
                ui.separator();
                ui.selectable_value(&mut state.tab, Tab::Main, "✉ Send Mail");
                ui.selectable_value(&mut state.tab, Tab::Settings, "⚙ Settings");
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
