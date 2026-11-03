use crate::keys::{
    MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, MOUSE_X1, MOUSE_X2, WHEEL_DOWN, WHEEL_LEFT, WHEEL_RIGHT,
    WHEEL_UP,
};
use crate::theme::{LINE, LINE_SOFT, SUNKEN};
use crate::ui_paint;
use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::collections::HashMap;

pub fn draw(ui: &mut Ui, counts: &HashMap<u16, u64>, height: f32) {
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
    painter.rect_filled(
        body.translate(Vec2::new(0.0, 6.0)),
        body_w * 0.42,
        Color32::from_black_alpha(90),
    );
    painter.rect(
        body,
        body_w * 0.42,
        SUNKEN,
        Stroke::new(1.0, LINE),
        StrokeKind::Inside,
    );
    let seam = body.top() + body_h * 0.34 + 4.0;
    painter.hline(
        (body.left() + body_w * 0.12)..=(body.right() - body_w * 0.12),
        seam,
        Stroke::new(1.0, LINE_SOFT),
    );
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
    let side_back = Rect::from_min_size(
        Pos2::new(side_x, side_top + side_h + 4.0),
        Vec2::new(side_w, side_h),
    );
    let max = peak(
        counts,
        &[
            MOUSE_LEFT,
            MOUSE_RIGHT,
            MOUSE_MIDDLE,
            MOUSE_X1,
            MOUSE_X2,
            WHEEL_UP,
            WHEEL_DOWN,
        ],
    );
    let count = |key| counts.get(&key).copied().unwrap_or(0);
    ui_paint::paint_key(
        ui,
        &painter,
        left,
        MOUSE_LEFT,
        "左键",
        count(MOUSE_LEFT),
        max,
    );
    ui_paint::paint_key(
        ui,
        &painter,
        right,
        MOUSE_RIGHT,
        "右键",
        count(MOUSE_RIGHT),
        max,
    );
    ui_paint::paint_key(
        ui,
        &painter,
        third(wheel, 0),
        WHEEL_UP,
        "↑",
        count(WHEEL_UP),
        max,
    );
    ui_paint::paint_key(
        ui,
        &painter,
        third(wheel, 2),
        WHEEL_DOWN,
        "↓",
        count(WHEEL_DOWN),
        max,
    );
    ui_paint::paint_key(
        ui,
        &painter,
        third(wheel, 1),
        MOUSE_MIDDLE,
        "中",
        count(MOUSE_MIDDLE),
        max,
    );
    ui_paint::paint_key(ui, &painter, side_front, MOUSE_X2, "侧2", count(MOUSE_X2), max);
    ui_paint::paint_key(ui, &painter, side_back, MOUSE_X1, "侧1", count(MOUSE_X1), max);
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

fn peak(counts: &HashMap<u16, u64>, keys: &[u16]) -> u64 {
    keys.iter()
        .map(|key| counts.get(key).copied().unwrap_or(0))
        .max()
        .unwrap_or(0)
}

fn third(rect: Rect, index: usize) -> Rect {
    let h = rect.height() / 3.0;
    let top = rect.top() + h * index as f32;
    Rect::from_min_max(
        Pos2::new(rect.left(), top + 1.5),
        Pos2::new(rect.right(), top + h - 1.5),
    )
}
