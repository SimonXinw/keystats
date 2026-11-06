use crate::stats::Snapshot;
use crate::theme::{self, ACCENT, CAPTION, FAINT, GAP, MUTED, NUMBER, OK, PAD, TEXT, TITLE, WARM};
use crate::ui_kit::{self, Tone};
use egui::{Align, Color32, Layout, RichText, Ui, Vec2};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct HeaderAction {
    pub start: bool,
    pub pause: bool,
    pub reset: bool,
    pub export: bool,
    pub settings: bool,
}

pub fn show(ui: &mut Ui, snap: &Snapshot, ready: bool, settings_open: bool) -> HeaderAction {
    let mut action = HeaderAction {
        start: false,
        pause: false,
        reset: false,
        export: false,
        settings: false,
    };
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        Vec2::new(width, 48.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui_kit::logo(ui, 44.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.horizontal(|ui| {
                    ui.label(RichText::new("按键统计").font(theme::bold(TITLE)).color(TEXT));
                    ui.label(RichText::new("keystats").size(TITLE - 4.0).color(FAINT));
                    ui.add_space(4.0);
                    if snap.running {
                        ui_kit::pill(ui, "统计中", OK);
                    } else {
                        ui_kit::pill(ui, "已暂停", WARM);
                    }
                });
                ui.label(ui_kit::caption(
                    "只读取按键次数，不注册快捷键，也不拦截你的操作。",
                ));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                action.export = ui_kit::button(ui, "导出表格", Tone::Plain, true).clicked();
                action.reset = ui_kit::button(ui, "清零", Tone::Plain, true).clicked();
                ui.add_space(GAP);
                action.pause = ui_kit::button(ui, "暂停", Tone::Plain, snap.running).clicked();
                let start = if snap.rows.is_empty() && snap.elapsed.is_zero() {
                    "开始"
                } else {
                    "继续"
                };
                action.start =
                    ui_kit::button(ui, start, Tone::Primary, !snap.running && ready).clicked();
                ui.add_space(GAP);
                let tone = if settings_open { Tone::Primary } else { Tone::Plain };
                action.settings = ui_kit::button(ui, "设置", tone, true)
                    .on_hover_text("关闭窗口时缩到托盘，或直接退出。")
                    .clicked();
            });
        },
    );
    ui.add_space(GAP + 4.0);
    cards(ui, snap);
    action
}

fn cards(ui: &mut Ui, snap: &Snapshot) {
    let total = snap.keyboard + snap.mouse;
    let pace = rate_note(total, snap.recent, snap.elapsed);
    let processes: HashSet<&str> = snap.rows.iter().map(|r| r.process.as_str()).collect();
    let items = [
        ("统计时长", format_duration(snap.elapsed), status(snap.running), TEXT),
        ("合计", total.to_string(), pace, ACCENT),
        ("键盘", snap.keyboard.to_string(), share(snap.keyboard, total), TEXT),
        ("鼠标", snap.mouse.to_string(), share(snap.mouse, total), TEXT),
        ("程序", processes.len().to_string(), "按前台进程分开记".to_owned(), TEXT),
    ];
    let count = items.len() as f32;
    let width = ((ui.available_width() - GAP * (count - 1.0)) / count).floor();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        for (title, value, note, color) in items {
            card(ui, width, title, &value, &note, color);
        }
    });
}

fn card(ui: &mut Ui, width: f32, title: &str, value: &str, note: &str, color: Color32) {
    ui_kit::card().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.set_width(width - PAD as f32 * 2.0 - 2.0);
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.label(RichText::new(title).size(CAPTION).color(MUTED));
            ui.label(RichText::new(value).font(theme::bold(NUMBER)).color(color));
            ui.label(RichText::new(note).size(CAPTION).color(FAINT));
        });
    });
}

fn rate_note(total: u64, recent: u64, elapsed: Duration) -> String {
    let recent = if total == 0 && recent == 0 {
        "—".into()
    } else {
        recent.to_string()
    };
    let minutes = elapsed.as_secs_f64() / 60.0;
    let average = if minutes >= 1.0 / 60.0 && total > 0 {
        format!("{:.0}", total as f64 / minutes)
    } else {
        "—".into()
    };
    format!("近一分钟 {recent} · 平均 {average}")
}

fn share(part: u64, total: u64) -> String {
    if total == 0 {
        "占比 —".to_owned()
    } else {
        format!("占比 {:.0}%", part as f64 * 100.0 / total as f64)
    }
}

fn status(running: bool) -> String {
    if running { "正在记录" } else { "暂停中，不记录" }.to_owned()
}

pub fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}
