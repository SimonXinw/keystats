use crate::board::{self, Spot};
use crate::keys::{self, is_mouse};
use crate::palette::Palette;
use crate::stats::Row;
use crate::theme::{CAPTION, FAINT, GAP};
use crate::ui_kit;
use crate::ui_mouse;
use crate::ui_paint::{self, Cap};
use crate::ui_palette::Colors;
use egui::{Align, Layout, Pos2, Rect, RichText, Sense, Ui, Vec2};
use std::collections::HashMap;

pub fn show(ui: &mut Ui, rows: &[Row], needle: &str, colors: &mut Colors) {
    let counts = collect(rows, needle);
    let palette = colors.palette;
    ui.horizontal(|ui| {
        ui.label(ui_kit::caption(
            "按住的键常亮，松开后渐暗；粘滞键激活时一直亮着。悬停可看完整次数。",
        ));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            colors.button(ui);
            ui.add_space(6.0);
            ui.label(RichText::new("多").size(CAPTION).color(FAINT));
            ui_paint::gradient(ui, &palette, Vec2::new(132.0, 10.0));
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
            draw_board(ui, &spots, &counts, scale, &palette);
        });
        ui.vertical(|ui| {
            ui.set_width(mouse_w);
            ui_kit::section(ui, "鼠标", &format!("{mouse} 次"));
            ui.add_space(4.0);
            ui_mouse::draw(ui, &counts, board_h * scale, &palette);
        });
    });
    if let Some(extra) = ui_mouse::extra_line(&counts) {
        ui.add_space(8.0);
        ui.label(ui_kit::caption(extra));
    }
}

fn draw_board(
    ui: &mut Ui,
    spots: &[Spot],
    counts: &HashMap<u16, u64>,
    scale: f32,
    palette: &Palette,
) {
    let (board_w, board_h) = board::bounds(spots);
    let (area, painter) =
        ui.allocate_painter(Vec2::new(board_w * scale, board_h * scale), Sense::empty());
    let origin = area.rect.min;
    let gap = (scale * 0.14).max(2.0);
    let caps: Vec<Cap> = spots
        .iter()
        .map(|spot| Cap {
            rect: Rect::from_min_size(
                Pos2::new(origin.x + spot.x * scale, origin.y + spot.y * scale),
                Vec2::new((spot.w * scale - gap).max(4.0), (spot.h * scale - gap).max(4.0)),
            ),
            key: spot.key,
            face: spot.label,
            count: counts.get(&spot.key).copied().unwrap_or(0),
        })
        .collect();
    let max = caps.iter().map(|cap| cap.count).max().unwrap_or(0);
    ui_paint::paint_all(ui, &painter, &caps, max, palette);
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
