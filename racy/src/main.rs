use eframe::egui;

use crate::app::FlameGraphApp;

pub mod app;
pub mod data;
pub mod event;
pub mod widget;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };

    eframe::run_native(
        "Process Event Flamegraphs",
        options,
        Box::new(|_cc| Ok(Box::new(FlameGraphApp::default()))),
    )
}
