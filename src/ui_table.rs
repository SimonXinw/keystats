use crate::stats::Snapshot;
use crate::theme::{CAPTION, FAINT, GAP, MUTED, TEXT};
use crate::ui_heat;
use crate::ui_kit;
use crate::ui_list;
use crate::ui_palette::Colors;
use egui::{Align, Color32, Layout, RichText, ScrollArea, TextEdit, Ui, Vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReportView {
    Detail,
    Process,
    Heat,
}

pub struct Status<'a> {
    pub text: &'a str,
    pub color: Color32,
}

pub fn show(
    ui: &mut Ui,
    filter: &mut String,
    view: &mut ReportView,
    count_self: &mut bool,
    snap: &Snapshot,
    status: Option<Status>,
    colors: &mut Colors,
) {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        Vec2::new(width, 40.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui_kit::segmented(
                ui,
                view,
                &[
                    (ReportView::Detail, "按键明细"),
                    (ReportView::Process, "进程汇总"),
                    (ReportView::Heat, "键位热力"),
                ],
            );
            ui.add_space(8.0);
            let label = if *count_self {
                "取消计入本程序"
            } else {
                "打开计入本程序"
            };
            if ui_kit::toggle(ui, *count_self, label) {
                *count_self = !*count_self;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    TextEdit::singleline(filter)
                        .hint_text("输入进程名或按键名")
                        .desired_width(260.0)
                        .margin(egui::Margin::symmetric(12, 8)),
                );
                ui.label(RichText::new("筛选").color(MUTED));
            });
        },
    );
    ui.add_space(GAP - 2.0);
    let needle = filter.trim().to_lowercase();
    let height = (ui.available_height() - 34.0).max(200.0);
    ui_kit::card().show(ui, |ui| {
        let inner = height - 34.0;
        ui.set_min_size(Vec2::new(ui.available_width(), inner));
        ui.set_max_height(inner);
        if snap.rows.is_empty() && *view != ReportView::Heat {
            empty(ui, inner);
            return;
        }
        match *view {
            ReportView::Detail => ui_list::detail(ui, &snap.rows, &needle),
            ReportView::Process => ui_list::process(ui, &snap.rows, &needle),
            ReportView::Heat => {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| ui_heat::show(ui, &snap.rows, &needle, colors));
            }
        }
    });
    ui.add_space(8.0);
    footer(ui, snap.dropped, status);
}

fn empty(ui: &mut Ui, height: f32) {
    ui.vertical_centered(|ui| {
        ui.add_space((height * 0.32).max(24.0));
        ui.label(RichText::new("还没有记录").size(18.0).color(TEXT));
        ui.add_space(4.0);
        ui.label(
            RichText::new("点「开始」后正常使用电脑或打游戏，回到这里就能看到每个键的次数。")
                .color(MUTED),
        );
    });
}

fn footer(ui: &mut Ui, dropped: u64, status: Option<Status>) {
    ui.horizontal(|ui| {
        if let Some(status) = status {
            ui.label(RichText::new(status.text).size(CAPTION).color(status.color));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let mut tip = "鼠标移动不计数 · 按住一个键只算一次".to_owned();
            if dropped > 0 {
                tip = format!("有 {dropped} 次因缓冲满被跳过 · {tip}");
            }
            ui.label(RichText::new(tip).size(CAPTION).color(FAINT));
        });
    });
}
