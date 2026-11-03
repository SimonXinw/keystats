use crate::keys;
use crate::theme::{self, FAINT, LINE_SOFT, RAISED, TEXT};
use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

const COLD: Color32 = Color32::from_rgb(28, 66, 86);
const MID: Color32 = Color32::from_rgb(40, 182, 170);
const HOT: Color32 = Color32::from_rgb(244, 188, 92);

pub fn paint_key(
    ui: &Ui,
    painter: &egui::Painter,
    rect: Rect,
    key: u16,
    face: &str,
    count: u64,
    max: u64,
) {
    let response = ui.interact(rect, ui.id().with(key), Sense::hover());
    let hot = response.hovered();
    let level = level(count, max);
    let radius = (rect.height().min(rect.width()) * 0.14).clamp(3.0, 9.0);
    let fill = match level {
        Some(t) => ramp(t),
        None => RAISED,
    };
    let fill = if hot { mix(fill, Color32::WHITE, 0.16) } else { fill };
    let depth = (rect.height() * 0.06).clamp(1.5, 3.5);
    painter.rect_filled(
        rect.translate(Vec2::new(0.0, depth)),
        radius,
        mix(fill, Color32::BLACK, 0.55),
    );
    let line = if hot {
        Stroke::new(1.5, TEXT)
    } else if level.is_some() {
        Stroke::NONE
    } else {
        Stroke::new(1.0, LINE_SOFT)
    };
    painter.rect(rect, radius, fill, line, StrokeKind::Inside);
    let ink = ink(level);
    let font = face_font(rect, face);
    let two_lines = count > 0 && rect.height() >= 34.0;
    let y = if two_lines {
        rect.center().y - font * 0.5
    } else {
        rect.center().y
    };
    painter.text(
        Pos2::new(rect.center().x, y),
        egui::Align2::CENTER_CENTER,
        face,
        FontId::proportional(font),
        ink,
    );
    if two_lines {
        painter.text(
            Pos2::new(rect.center().x, rect.center().y + font * 0.55),
            egui::Align2::CENTER_CENTER,
            short_count(count),
            theme::bold((font * 0.9).max(8.0)),
            ink,
        );
    }
    if hot {
        response.on_hover_text(format!("{}\n{} 次", keys::label(key), count));
    }
}

pub fn ramp(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.6 {
        mix(COLD, MID, t / 0.6)
    } else {
        mix(MID, HOT, (t - 0.6) / 0.4)
    }
}

fn level(count: u64, max: u64) -> Option<f32> {
    if count == 0 || max == 0 {
        None
    } else {
        Some((count as f32 / max as f32).sqrt().clamp(0.06, 1.0))
    }
}

fn face_font(rect: Rect, face: &str) -> f32 {
    let by_height = (rect.height().min(rect.width()) * 0.32).max(8.0);
    let units: f32 = face
        .chars()
        .map(|c| if c.is_ascii() { 0.62 } else { 1.1 })
        .sum::<f32>()
        .max(0.62);
    let by_width = rect.width() * 0.82 / units;
    by_height.min(by_width).max(7.0)
}

fn ink(level: Option<f32>) -> Color32 {
    match level {
        None => FAINT,
        Some(t) if t > 0.42 => Color32::from_rgb(10, 28, 30),
        Some(_) => TEXT,
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        lerp(a.r(), b.r(), t),
        lerp(a.g(), b.g(), t),
        lerp(a.b(), b.b(), t),
    )
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn short_count(count: u64) -> String {
    if count < 1000 {
        count.to_string()
    } else if count < 10_000 {
        format!("{:.1}k", count as f32 / 1000.0)
    } else {
        format!("{}k", count / 1000)
    }
}
