use crate::close_mode::CloseMode;
use crate::theme::{self, LINE, R_CARD, SURFACE, TEXT};
use crate::ui_kit::{self, Tone};
use egui::{Align, Align2, Frame, Layout, RichText, Stroke, Vec2};

enum Panel {
    Stay,
    Done,
    Quit,
}

pub fn show(ctx: &egui::Context, open: &mut bool, mode: &mut CloseMode) -> bool {
    if !*open {
        return false;
    }
    let mut panel = Panel::Stay;
    let mut shown = true;
    let frame = Frame::window(&ctx.style())
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(R_CARD + 2)
        .inner_margin(18);
    egui::Window::new(RichText::new("设置").font(theme::bold(16.0)).color(TEXT))
        .id(egui::Id::new("settings"))
        .open(&mut shown)
        .collapsible(false)
        .resizable(false)
        .frame(frame)
        .pivot(Align2::RIGHT_TOP)
        .default_pos(ctx.screen_rect().right_top() + Vec2::new(-36.0, 88.0))
        .show(ctx, |ui| {
            panel = body(ui, mode);
        });
    match panel {
        Panel::Quit => {
            *open = false;
            true
        }
        Panel::Done => {
            *open = false;
            false
        }
        Panel::Stay => {
            *open = shown;
            false
        }
    }
}

fn body(ui: &mut egui::Ui, mode: &mut CloseMode) -> Panel {
    ui.set_width(328.0);
    ui.label(ui_kit::caption("改动会保存在本机，下次打开仍然有效。"));
    ui.add_space(12.0);
    ui.label(
        RichText::new("关闭窗口")
            .font(theme::bold(15.0))
            .color(TEXT),
    );
    ui.add_space(2.0);
    ui.label(ui_kit::caption("右上角关闭和 Alt+F4 都按这里执行。"));
    ui.add_space(8.0);
    ui_kit::segmented(
        ui,
        mode,
        &[(CloseMode::Tray, "缩到托盘"), (CloseMode::Quit, "直接退出")],
    );
    ui.add_space(8.0);
    ui.label(ui_kit::caption(hint(*mode)));
    ui.add_space(14.0);
    ui.label(ui_kit::caption(
        "退出会卸掉钩子。已经记下的次数会留在本机。",
    ));
    ui.add_space(8.0);
    let mut panel = Panel::Stay;
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), 36.0),
        Layout::right_to_left(Align::Center),
        |ui| {
            if ui_kit::button(ui, "完成", Tone::Primary, true).clicked() {
                panel = Panel::Done;
            }
            ui.add_space(8.0);
            if ui_kit::button(ui, "退出程序", Tone::Plain, true).clicked() {
                panel = Panel::Quit;
            }
        },
    );
    panel
}

fn hint(mode: CloseMode) -> &'static str {
    match mode {
        CloseMode::Tray => "窗口会藏进右下角托盘，计数继续。单击图标打开，右键可以退出。",
        CloseMode::Quit => "窗口关掉后程序结束，键盘和鼠标钩子会卸掉。",
    }
}
