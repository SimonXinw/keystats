use crate::theme::{self, BODY, DANGER, LINE, R_CARD, SURFACE, TEXT};
use crate::ui_kit::{self, Tone};
use egui::{Align, Color32, Frame, Id, Layout, Modal, RichText, Stroke};

pub enum ResetChoice {
    Stay,
    Cancel,
    Confirm,
}

pub fn confirm_reset(ctx: &egui::Context) -> ResetChoice {
    let mut choice = ResetChoice::Stay;
    let frame = Frame::popup(&ctx.style())
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(R_CARD + 4)
        .inner_margin(24);
    let shown = Modal::new(Id::new("reset_confirm"))
        .backdrop_color(Color32::from_black_alpha(170))
        .frame(frame)
        .show(ctx, |ui| {
            ui.set_width(420.0);
            ui_kit::pill(ui, "不可撤销", DANGER);
            ui.add_space(12.0);
            ui.label(RichText::new("清零当前统计？").font(theme::bold(22.0)).color(TEXT));
            ui.add_space(6.0);
            ui.label(
                RichText::new("所有进程的次数和计时都会清掉，之后要重新点「开始」。")
                    .size(BODY)
                    .color(TEXT),
            );
            ui.label(ui_kit::caption("已经导出的表格文件不会受影响。按 Esc 可取消。"));
            ui.add_space(20.0);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 36.0),
                Layout::right_to_left(Align::Center),
                |ui| {
                    if ui_kit::button(ui, "确认清零", Tone::Danger, true).clicked() {
                        choice = ResetChoice::Confirm;
                    }
                    if ui_kit::button(ui, "取消", Tone::Plain, true).clicked() {
                        choice = ResetChoice::Cancel;
                    }
                },
            );
        });
    if shown.should_close() && matches!(choice, ResetChoice::Stay) {
        ResetChoice::Cancel
    } else {
        choice
    }
}
