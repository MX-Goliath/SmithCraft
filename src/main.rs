#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() -> eframe::Result {
    if std::env::args().any(|arg| arg == "--version") {
        println!("SmithCraft {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let options = eframe::NativeOptions {
        persistence_path: smithcraft::runtime::persistence_path()
            .map_err(|error| eframe::Error::AppCreation(Box::new(error)))?,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1500., 940.])
            .with_maximized(true)
            .with_min_inner_size([800., 600.]),
        ..Default::default()
    };
    eframe::run_native(
        "Smith V4.1 — Rust",
        options,
        Box::new(|cc| Ok(Box::new(smithcraft::app::SmithApp::new(cc)))),
    )
}
