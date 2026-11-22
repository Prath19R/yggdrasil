mod app;

use app::YggdrasilApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Start multithreaded Tokio runtime for background AI streaming and terminal PTY
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime");

    let _enter = rt.enter();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Yggdrasil — Pure Rust AI IDE")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Yggdrasil",
        native_options,
        Box::new(|cc| Ok(Box::new(YggdrasilApp::new(cc)))),
    )
}
