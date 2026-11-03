use crate::keys::{self, category, is_mouse};
use crate::stats::Row;
use crate::theme::{self, ACCENT, BODY, CAPTION, FAINT, LINE_SOFT, MUTED, RAISED, SUNKEN, TEXT};
use egui::{Align2, Color32, FontId, Pos2, Rect, ScrollArea, Sense, Ui, Vec2};
use std::collections::BTreeMap;

const ROW_H: f32 = 36.0;
const HEAD_H: f32 = 30.0;

#[derive(Clone, Copy)]
enum Kind {
    Name,
    Strong,
    Tag,
    Total,
}

struct Col {
    title: &'static str,
    share: f32,
    right: bool,
    kind: Kind,
}

const DETAIL: [Col; 4] = [
    Col { title: "进程", share: 0.36, right: false, kind: Kind::Name },
    Col { title: "按键", share: 0.22, right: false, kind: Kind::Strong },
    Col { title: "类别", share: 0.12, right: false, kind: Kind::Tag },
    Col { title: "次数", share: 0.30, right: true, kind: Kind::Total },
];

const PROCESS: [Col; 4] = [
    Col { title: "进程", share: 0.34, right: false, kind: Kind::Strong },
    Col { title: "键盘", share: 0.18, right: true, kind: Kind::Name },
    Col { title: "鼠标", share: 0.18, right: true, kind: Kind::Name },
    Col { title: "合计", share: 0.30, right: true, kind: Kind::Total },
];

pub fn detail(ui: &mut Ui, rows: &[Row], needle: &str) {
    let list: Vec<(&Row, String)> = rows
        .iter()
        .map(|row| (row, keys::label(row.key)))
        .filter(|(row, name)| matches(needle, &row.process, Some(name)))
        .collect();
    let max = list.iter().map(|(row, _)| row.count).max().unwrap_or(0);
    table(ui, &DETAIL, list.len(), |i| {
        let (row, name) = &list[i];
        (
            [
                row.process.clone(),
                name.clone(),
                category(row.key).to_owned(),
                row.count.to_string(),
            ],
            row.count,
            max,
        )
    });
}

pub fn process(ui: &mut Ui, rows: &[Row], needle: &str) {
    let mut sums: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
    for row in rows {
        if !matches(needle, &row.process, None) {
            continue;
        }
        let slot = sums.entry(row.process.as_str()).or_default();
        if is_mouse(row.key) {
            slot.1 = slot.1.saturating_add(row.count);
        } else {
            slot.0 = slot.0.saturating_add(row.count);
        }
    }
    let mut list: Vec<_> = sums.into_iter().collect();
    list.sort_by(|a, b| (b.1 .0 + b.1 .1).cmp(&(a.1 .0 + a.1 .1)));
    let max = list.iter().map(|(_, (k, m))| k + m).max().unwrap_or(0);
    table(ui, &PROCESS, list.len(), |i| {
        let (name, (keyboard, mouse)) = list[i];
        let total = keyboard + mouse;
        (
            [
                name.to_owned(),
                keyboard.to_string(),
                mouse.to_string(),
                total.to_string(),
            ],
            total,
            max,
        )
    });
}

fn table(ui: &mut Ui, cols: &[Col; 4], len: usize, cell: impl Fn(usize) -> ([String; 4], u64, u64)) {
    let width = ui.available_width();
    let (head, _) = ui.allocate_exact_size(Vec2::new(width, HEAD_H), Sense::hover());
    for (col, rect) in cols.iter().zip(columns(head, cols)) {
        put(ui, rect, col.title, col.right, theme::bold(CAPTION), MUTED);
    }
    ui.painter().hline(head.x_range(), head.bottom(), (1.0, LINE_SOFT));
    if len == 0 {
        ui.add_space(16.0);
        ui.label(egui::RichText::new("没有符合筛选条件的记录。").color(MUTED));
        return;
    }
    ui.spacing_mut().item_spacing.y = 0.0;
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, ROW_H, len, |ui, range| {
            let width = ui.available_width();
            for i in range {
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, ROW_H), Sense::hover());
                let fill = if resp.hovered() {
                    RAISED
                } else if i % 2 == 1 {
                    SUNKEN.gamma_multiply(0.55)
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter().rect_filled(rect.shrink2(Vec2::new(0.0, 1.0)), 6.0, fill);
                let (texts, value, max) = cell(i);
                let rects = columns(rect, cols);
                bar(ui, rects[3], value, max);
                for (c, text) in texts.iter().enumerate() {
                    let (font, color) = match cols[c].kind {
                        Kind::Name => (FontId::proportional(BODY), MUTED),
                        Kind::Tag => (FontId::proportional(CAPTION), FAINT),
                        Kind::Total => (theme::bold(BODY + 1.0), TEXT),
                        Kind::Strong => (theme::bold(BODY), TEXT),
                    };
                    put(ui, rects[c], text, cols[c].right, font, color);
                }
            }
        });
}

fn columns(rect: Rect, cols: &[Col; 4]) -> [Rect; 4] {
    let mut x = rect.left();
    let mut out = [rect; 4];
    for (slot, col) in out.iter_mut().zip(cols) {
        let w = rect.width() * col.share;
        *slot = Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(w, rect.height()));
        x += w;
    }
    out
}

fn put(ui: &Ui, rect: Rect, text: &str, right: bool, font: FontId, color: Color32) {
    let cell = rect.shrink2(Vec2::new(12.0, 0.0));
    let (pos, align) = if right {
        (cell.right_center(), Align2::RIGHT_CENTER)
    } else {
        (cell.left_center(), Align2::LEFT_CENTER)
    };
    ui.painter()
        .with_clip_rect(cell.intersect(ui.clip_rect()))
        .text(pos, align, text, font, color);
}

fn bar(ui: &Ui, rect: Rect, value: u64, max: u64) {
    let track_w = rect.width() - 100.0;
    if track_w < 24.0 || max == 0 {
        return;
    }
    let track = Rect::from_min_size(
        Pos2::new(rect.left() + 12.0, rect.center().y - 3.0),
        Vec2::new(track_w, 6.0),
    );
    let painter = ui.painter();
    painter.rect_filled(track, 3.0, LINE_SOFT);
    let t = (value as f32 / max as f32).clamp(0.0, 1.0);
    let fill = Rect::from_min_size(track.min, Vec2::new((track_w * t).max(3.0), 6.0));
    painter.rect_filled(fill, 3.0, ACCENT.gamma_multiply(0.45 + 0.55 * t));
}

fn matches(needle: &str, process: &str, key_name: Option<&str>) -> bool {
    if needle.is_empty() {
        return true;
    }
    process.to_lowercase().contains(needle)
        || key_name.is_some_and(|name| name.to_lowercase().contains(needle))
}
