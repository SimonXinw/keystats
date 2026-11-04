use crate::app::KeyStatsApp;
use crate::theme;

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(crate::instance::TITLE)
            .with_icon(app_icon())
            .with_inner_size([1500.0, 880.0])
            .with_min_inner_size([1280.0, 680.0]),
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        crate::instance::TITLE,
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(KeyStatsApp::new()))
        }),
    )
}

fn app_icon() -> egui::IconData {
    egui::IconData {
        rgba: include_bytes!("../assets/icon_64.rgba").to_vec(),
        width: 64,
        height: 64,
    }
}
