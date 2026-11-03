use crate::keys::{MOUSE_LEFT, MOUSE_RIGHT, MOUSE_X1, MOUSE_X2, WHEEL_DOWN, WHEEL_UP};
use crate::stats::Stats;
use crate::ui_table::ReportView;
use std::sync::Arc;

pub fn view() -> Option<ReportView> {
    match std::env::var("KEYSTATS_DEMO").ok()?.as_str() {
        "process" => Some(ReportView::Process),
        "heat" => Some(ReportView::Heat),
        _ => None,
    }
}

pub fn reset_open() -> bool {
    std::env::var("KEYSTATS_DEMO").is_ok_and(|v| v == "reset")
}

pub fn capture(ctx: &egui::Context) {
    let Some(path) = std::env::var_os("KEYSTATS_SHOT") else {
        return;
    };
    let frame = ctx.cumulative_pass_nr();
    if frame == 20 {
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
    }
    ctx.request_repaint();
    let shot = ctx.input(|i| {
        i.raw.events.iter().find_map(|e| match e {
            egui::Event::Screenshot { image, .. } => Some(image.clone()),
            _ => None,
        })
    });
    if let Some(image) = shot {
        let mut bytes = Vec::with_capacity(image.pixels.len() * 4 + 8);
        bytes.extend((image.size[0] as u32).to_le_bytes());
        bytes.extend((image.size[1] as u32).to_le_bytes());
        for px in &image.pixels {
            bytes.extend(px.to_array());
        }
        let _ = std::fs::write(path, bytes);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

pub fn seed(stats: &Stats) {
    if std::env::var_os("KEYSTATS_DEMO").is_none() {
        return;
    }
    stats.start();
    let session = stats.session();
    let game: Arc<str> = Arc::from("League of Legends.exe");
    let editor: Arc<str> = Arc::from("Code.exe");
    let plan: [(&Arc<str>, u16, u32); 16] = [
        (&game, b'Q' as u16, 1840),
        (&game, b'W' as u16, 960),
        (&game, b'E' as u16, 1320),
        (&game, b'R' as u16, 310),
        (&game, b'D' as u16, 140),
        (&game, b'F' as u16, 120),
        (&game, b'1' as u16, 60),
        (&game, 0x20, 220),
        (&game, MOUSE_RIGHT, 5200),
        (&game, MOUSE_LEFT, 2100),
        (&game, MOUSE_X1, 7),
        (&game, MOUSE_X2, 4),
        (&editor, b'S' as u16, 420),
        (&editor, 0x0D, 380),
        (&editor, WHEEL_DOWN, 900),
        (&editor, WHEEL_UP, 640),
    ];
    for (name, key, times) in plan {
        for _ in 0..times {
            stats.ingest(session, Arc::clone(name), key);
        }
    }
}
