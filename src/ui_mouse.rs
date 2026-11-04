use crate::keys::{
    MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, MOUSE_X1, MOUSE_X2, WHEEL_DOWN, WHEEL_LEFT, WHEEL_RIGHT,
    WHEEL_UP,
};
use crate::palette::Palette;
use crate::theme::{LINE, LINE_SOFT, SUNKEN};
use crate::ui_paint::{self, Cap};
use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::collections::HashMap;

pub fn draw(ui: &mut Ui, counts: &HashMap<u16, u64>, height: f32, palette: &Palette) {
    let width = ui.available_width();
    let (area, painter) = ui.allocate_painter(Vec2::new(width, height.max(180.0)), Sense::empty());
    let max_h = (area.rect.height() * 0.92).max(160.0);
    let max_w = (width - 8.0).max(72.0);
    let body_h = max_h.min(max_w / 0.58);
    let body_w = body_h * 0.58;
    let body = Rect::from_center_size(
        Pos2::new(area.rect.center().x, area.rect.top() + body_h * 0.5 + 4.0),
        Vec2::new(body_w, body_h),
    );
    shell(&painter, body);
    let split = body.top() + body_h * 0.34;
    let mid = body.center().x;
    let left = Rect::from_min_max(
        Pos2::new(body.left() + 8.0, body.top() + 10.0),
        Pos2::new(mid - 3.0, split),
    );
    let right = Rect::from_min_max(
        Pos2::new(mid + 3.0, body.top() + 10.0),
        Pos2::new(body.right() - 8.0, split),
    );
    let wheel = Rect::from_center_size(
        Pos2::new(mid, split + body_h * 0.15),
        Vec2::new(body_w * 0.24, body_h * 0.30),
    );
    let side_w = (body_w * 0.24).max(22.0);
    let side_h = (body_h * 0.10).max(18.0);
    let side_x = body.left() + body_w * 0.05;
    let side_top = split + body_h * 0.06;
    let side_front = Rect::from_min_size(Pos2::new(side_x, side_top), Vec2::new(side_w, side_h));
    let side_back = side_front.translate(Vec2::new(0.0, side_h + 4.0));
    let cap = |rect: Rect, key: u16, face: &'static str| Cap {
        rect,
        key,
        face,
        count: counts.get(&key).copied().unwrap_or(0),
    };
    let caps = [
        cap(left, MOUSE_LEFT, "左键"),
        cap(right, MOUSE_RIGHT, "右键"),
        cap(third(wheel, 0), WHEEL_UP, "↑"),
        cap(third(wheel, 1), MOUSE_MIDDLE, "中"),
        cap(third(wheel, 2), WHEEL_DOWN, "↓"),
        cap(side_front, MOUSE_X2, "侧2"),
        cap(side_back, MOUSE_X1, "侧1"),
    ];
    let max = caps.iter().map(|cap| cap.count).max().unwrap_or(0);
    ui_paint::paint_all(ui, &painter, &caps, max, palette);
}

pub fn extra_line(counts: &HashMap<u16, u64>) -> Option<String> {
    let parts = [(WHEEL_LEFT, "滚轮左"), (WHEEL_RIGHT, "滚轮右")];
    let text: Vec<_> = parts
        .iter()
        .filter_map(|(key, name)| {
            let count = counts.get(key).copied().unwrap_or(0);
            (count > 0).then(|| format!("{name} {count}"))
        })
        .collect();
    if text.is_empty() {
        None
    } else {
        Some(format!("未画在图上：{}", text.join("，")))
    }
}

/// 机身：落地阴影 + 底边厚度 + 顶部一层很淡的高光。
fn shell(painter: &egui::Painter, body: Rect) {
    let radius = body.width() * 0.42;
    painter.rect_filled(
        body.translate(Vec2::new(0.0, 8.0)),
        radius,
        Color32::from_black_alpha(90),
    );
    painter.rect_filled(
        body.translate(Vec2::new(0.0, 4.0)),
        radius,
        Color32::from_rgb(6, 8, 12),
    );
    painter.rect(body, radius, SUNKEN, Stroke::new(1.0, LINE), StrokeKind::Inside);
    ui_paint::sheen(painter, body, radius, 0.6);
    let seam = body.top() + body.height() * 0.34 + 4.0;
    painter.hline(
        (body.left() + body.width() * 0.12)..=(body.right() - body.width() * 0.12),
        seam,
        Stroke::new(1.0, LINE_SOFT),
    );
}

fn third(rect: Rect, index: usize) -> Rect {
    let h = rect.height() / 3.0;
    let top = rect.top() + h * index as f32;
    Rect::from_min_max(
        Pos2::new(rect.left(), top + 1.5),
        Pos2::new(rect.right(), top + h - 1.5),
    )
}
