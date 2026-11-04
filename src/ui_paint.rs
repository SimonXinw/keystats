use crate::keys;
use crate::live;
use crate::palette::{ink_on, mix, Palette};
use crate::theme::{self, FAINT, LINE_SOFT, RAISED, TEXT};
use egui::epaint::Shadow;
use egui::{Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

pub struct Cap<'a> {
    pub rect: Rect,
    pub key: u16,
    pub face: &'a str,
    pub count: u64,
}

/// 先铺所有亮键的柔光，再画键帽，光只从键缝里透出来，不会盖住旁边的字。
pub fn paint_all(ui: &Ui, painter: &Painter, caps: &[Cap], max: u64, palette: &Palette) {
    let glows: Vec<f32> = caps.iter().map(|cap| live::glow(cap.key)).collect();
    for (cap, glow) in caps.iter().zip(&glows) {
        if *glow > 0.01 {
            halo(painter, cap.rect, *glow, palette.lamp);
        }
    }
    for (cap, glow) in caps.iter().zip(&glows) {
        paint_key(ui, painter, cap, *glow, max, palette);
    }
}

pub fn gradient(ui: &mut Ui, palette: &Palette, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let steps = 24;
    let step = rect.width() / steps as f32;
    for i in 0..steps {
        let t = (i as f32 + 0.5) / steps as f32;
        let cell = Rect::from_min_size(
            Pos2::new(rect.left() + step * i as f32, rect.top()),
            Vec2::new(step + 0.5, rect.height()),
        );
        ui.painter().rect_filled(cell, 0.0, palette.ramp(t));
    }
}

fn halo(painter: &Painter, rect: Rect, glow: f32, lamp: Color32) {
    let blur = (rect.height() * 0.62).clamp(10.0, 22.0);
    let shadow = Shadow {
        offset: [0, 0],
        blur: blur as u8,
        spread: (blur * 0.08) as u8,
        color: lamp.gamma_multiply(0.38 * glow),
    };
    painter.add(shadow.as_shape(rect, radius_of(rect)));
}

fn paint_key(ui: &Ui, painter: &Painter, cap: &Cap, glow: f32, max: u64, palette: &Palette) {
    let response = ui.interact(cap.rect, ui.id().with(cap.key), Sense::hover());
    let hot = response.hovered();
    let level = level(cap.count, max);
    let base = level.map_or(RAISED, |t| palette.ramp(t));
    let fill = mix(base, palette.lamp, glow);
    let blank = level.is_none() && glow <= 0.01;
    let top = keycap(painter, cap.rect, fill, glow, hot, blank);
    let ink = if blank { FAINT } else { ink_on(fill) };
    label(painter, top, cap, ink);
    if hot {
        response.on_hover_text(format!("{}\n{} 次", keys::label(cap.key), cap.count));
    }
}

/// 键面顶边和外轮廓齐平，厚度只在左右和底部。
/// 柔光先铺在外轮廓上，这层实心裙边把向内的光盖住，所以光只从最外圈透出去。
fn keycap(
    painter: &Painter,
    rect: Rect,
    fill: Color32,
    glow: f32,
    hot: bool,
    blank: bool,
) -> Rect {
    let depth = (rect.height() * 0.14).clamp(2.6, 5.2);
    let inset = (depth * 0.62).clamp(1.8, 3.6);
    let radius = radius_of(rect);
    let pressed = depth * 0.45 * glow;
    let top = Rect::from_min_max(
        Pos2::new(rect.left() + inset, rect.top()),
        Pos2::new(rect.right() - inset, rect.bottom() - depth + pressed),
    );
    let wall = mix(fill, Color32::BLACK, 0.24 - 0.08 * glow);
    painter.rect_filled(rect, radius, wall);
    let face = if hot { mix(fill, Color32::WHITE, 0.08) } else { fill };
    painter.rect_filled(top, radius, face);
    sheen(painter, top, radius, 1.0 - glow);
    if hot && glow < 0.35 {
        painter.rect_stroke(top, radius, Stroke::new(1.5, TEXT), StrokeKind::Inside);
    } else if blank {
        painter.rect_stroke(top, radius, Stroke::new(1.0, LINE_SOFT), StrokeKind::Inside);
    }
    top
}

/// 几层很淡的白叠出从上往下的渐变，比一刀切的半边高光柔和。
pub fn sheen(painter: &Painter, top: Rect, radius: f32, strength: f32) {
    let alpha = (5.0 * strength) as u8;
    if alpha == 0 {
        return;
    }
    for share in [0.72, 0.48, 0.26] {
        let band = Rect::from_min_max(
            top.min,
            Pos2::new(top.right(), top.top() + top.height() * share),
        );
        painter.rect_filled(band, radius, white(alpha));
    }
}

/// 按显示色值直接给的半透明白。`Color32::from_white_alpha` 走线性空间，实际会亮好几倍。
pub fn white(alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(alpha, alpha, alpha, alpha)
}

fn label(painter: &Painter, top: Rect, cap: &Cap, ink: Color32) {
    let font = face_font(top, cap.face);
    let two_lines = cap.count > 0 && top.height() >= 30.0;
    let y = if two_lines {
        top.center().y - font * 0.5
    } else {
        top.center().y
    };
    painter.text(
        Pos2::new(top.center().x, y),
        egui::Align2::CENTER_CENTER,
        cap.face,
        FontId::proportional(font),
        ink,
    );
    if two_lines {
        painter.text(
            Pos2::new(top.center().x, top.center().y + font * 0.55),
            egui::Align2::CENTER_CENTER,
            short_count(cap.count),
            theme::bold((font * 0.9).max(8.0)),
            ink,
        );
    }
}

fn radius_of(rect: Rect) -> f32 {
    (rect.height().min(rect.width()) * 0.14).clamp(3.0, 9.0)
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

fn short_count(count: u64) -> String {
    if count < 1000 {
        count.to_string()
    } else if count < 10_000 {
        format!("{:.1}k", count as f32 / 1000.0)
    } else {
        format!("{}k", count / 1000)
    }
}
