mod app;
mod canvas;
mod rendering;
mod ui;
mod game;
mod blockchain;
mod utils;
mod mode;
mod content_filter;
mod billboard;

use app::AlepicApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Alepic - Alephium Collaborative Canvas",
        native_options,
        Box::new(|cc| Ok(Box::new(AlepicApp::new(cc)))),
    )
}
