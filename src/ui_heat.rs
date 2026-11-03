use crate::board::{self, Spot};
use crate::keys::{self, is_mouse};
use crate::stats::Row;
use crate::theme::{CAPTION, FAINT, GAP};
use crate::ui_kit;
use crate::ui_mouse;
use crate::ui_paint;
use egui::{Align, Layout, Pos2, Rect, RichText, Sense, Ui, Vec2};
use std::collections::HashMap;

pub fn show(ui: &mut Ui, rows: &[Row], needle: &str) {
    let counts = collect(rows, needle);
    ui.horizontal(|ui| {
        ui.label(ui_kit::caption(
            "颜色越暖次数越多，悬停可看按键名和完整次数。筛选进程名后，只看该程序。",
        ));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new("多").size(CAPTION).color(FAINT));
            legend(ui);
            ui.label(RichText::new("少").size(CAPTION).color(FAINT));
        });
    });
    ui.add_space(GAP);
    let spots = board::keyboard();
    let (board_w, board_h) = board::bounds(&spots);
    let total = ui.available_width();
    let mouse_w = (total * 0.20).max(132.0);
    let key_w = (total - mouse_w - GAP * 2.0).max(240.0);
    let scale = (key_w / board_w).max(11.0);
    let (keyboard, mouse) = totals(&counts);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = GAP * 2.0;
        ui.vertical(|ui| {
            ui.set_width(key_w);
            ui_kit::section(ui, "键盘", &format!("{keyboard} 次"));
            ui.add_space(4.0);
            draw_board(ui, &spots, &counts, scale);
        });
        ui.vertical(|ui| {
            ui.set_width(mouse_w);
            ui_kit::section(ui, "鼠标", &format!("{mouse} 次"));
            ui.add_space(4.0);
            ui_mouse::draw(ui, &counts, board_h * scale);
        });
    });
    if let Some(extra) = ui_mouse::extra_line(&counts) {
        ui.add_space(8.0);
        ui.label(ui_kit::caption(extra));
    }
}

fn legend(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(132.0, 10.0), Sense::hover());
    let steps = 24;
    let step = rect.width() / steps as f32;
    let painter = ui.painter();
    for i in 0..steps {
        let t = (i as f32 + 0.5) / steps as f32;
        let cell = Rect::from_min_size(
            Pos2::new(rect.left() + step * i as f32, rect.top()),
            Vec2::new(step + 0.5, rect.height()),
        );
        painter.rect_filled(cell, 0.0, ui_paint::ramp(t));
    }
}

fn draw_board(ui: &mut Ui, spots: &[Spot], counts: &HashMap<u16, u64>, scale: f32) {
    let (board_w, board_h) = board::bounds(spots);
    let max = peak(counts, spots);
    let (area, painter) =
        ui.allocate_painter(Vec2::new(board_w * scale, board_h * scale), Sense::empty());
    let origin = area.rect.min;
    let gap = (scale * 0.14).max(2.0);
    for spot in spots {
        let rect = Rect::from_min_size(
            Pos2::new(origin.x + spot.x * scale, origin.y + spot.y * scale),
            Vec2::new((spot.w * scale - gap).max(4.0), (spot.h * scale - gap).max(4.0)),
        );
        let count = counts.get(&spot.key).copied().unwrap_or(0);
        ui_paint::paint_key(ui, &painter, rect, spot.key, spot.label, count, max);
    }
}

fn collect(rows: &[Row], needle: &str) -> HashMap<u16, u64> {
    let mut map = HashMap::new();
    for row in rows {
        if !needle.is_empty() {
            let name = keys::label(row.key);
            if !row.process.to_lowercase().contains(needle) && !name.to_lowercase().contains(needle)
            {
                continue;
            }
        }
        let slot = map.entry(row.key).or_insert(0u64);
        *slot = slot.saturating_add(row.count);
    }
    map
}

fn totals(counts: &HashMap<u16, u64>) -> (u64, u64) {
    counts.iter().fold((0, 0), |(k, m), (key, n)| {
        if is_mouse(*key) {
            (k, m.saturating_add(*n))
        } else {
            (k.saturating_add(*n), m)
        }
    })
}

fn peak(counts: &HashMap<u16, u64>, spots: &[Spot]) -> u64 {
    spots
        .iter()
        .map(|s| counts.get(&s.key).copied().unwrap_or(0))
        .max()
        .unwrap_or(0)
}
