mod models;
mod app;
mod api;
mod ui;

use app::MailApp;

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // env_logger::init(); // Requires env_logger dependency if used

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Auto Mail Pilot",
        native_options,
        Box::new(|cc| Ok(Box::new(MailApp::new(cc)))),
    )
}
