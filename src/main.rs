// Victoria Ambulance — native Windows app (Rust).
// No console window behind the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod auth;
mod chime;
mod core;
mod fieldguide;
mod relay;
mod settings;
mod storage;
mod timeclock;
mod ui;
mod update;

fn main() -> eframe::Result {
    let icon = image::load_from_memory(include_bytes!("../assets/icon.png")).ok().map(|img| {
        let img = img.to_rgba8();
        let (w, h) = img.dimensions();
        eframe::egui::IconData { rgba: img.into_raw(), width: w, height: h }
    });
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("Victoria Ambulance")
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([400.0, 480.0])
        .with_maximized(std::env::var("VA_WINDOWED").is_err());
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    let core = core::Core::new();
    eframe::run_native("Victoria Ambulance", options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, core)))))
}
