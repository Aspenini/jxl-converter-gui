mod app;
mod engine;
mod types;

use app::JxlConverterApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "JPEG XL Converter",
        options,
        Box::new(|_cc| Ok(Box::new(JxlConverterApp::new()))),
    )
}

