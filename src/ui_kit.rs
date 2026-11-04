use crate::theme::{
    self, ACCENT, ACCENT_INK, CAPTION, CTRL_H, DANGER, DANGER_INK, FAINT, LINE, LINE_SOFT, MUTED, PAD,
    RAISED, R_CARD, R_CTRL, SUNKEN, SURFACE, TEXT,
};
use egui::{Align2, Color32, Frame, Pos2, Response, RichText, Sense, Stroke, Ui, Vec2};

#[derive(Clone, Copy)]
pub enum Tone {
    Primary,
    Plain,
    Danger,
}

pub fn button(ui: &mut Ui, text: &str, tone: Tone, enabled: bool) -> Response {
    let (fill, ink, line) = match (tone, enabled) {
        (_, false) => (SURFACE, FAINT, LINE_SOFT),
        (Tone::Primary, true) => (ACCENT, ACCENT_INK, ACCENT),
        (Tone::Plain, true) => (RAISED, TEXT, LINE),
        (Tone::Danger, true) => (DANGER, DANGER_INK, DANGER),
    };
    let width = (text.chars().count() as f32 * 15.0 + 36.0).max(84.0);
    let widget = egui::Button::new(RichText::new(text).font(theme::bold(14.0)).color(ink))
        .fill(fill)
        .stroke(Stroke::new(1.0, line))
        .corner_radius(R_CTRL)
        .min_size(Vec2::new(width, CTRL_H));
    ui.add_enabled(enabled, widget)
}

pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) {
    Frame::NONE
        .fill(SUNKEN)
        .stroke(Stroke::new(1.0, LINE_SOFT))
        .corner_radius(R_CTRL + 2)
        .inner_margin(3)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.horizontal(|ui| {
                for (option, label) in options {
                    let on = *value == *option;
                    let text = RichText::new(*label).font(theme::bold(14.0)).color(if on {
                        ACCENT
                    } else {
                        MUTED
                    });
                    let widget = egui::Button::new(text)
                        .fill(if on { RAISED } else { Color32::TRANSPARENT })
                        .stroke(if on {
                            Stroke::new(1.0, LINE)
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(R_CTRL)
                        .min_size(Vec2::new(98.0, CTRL_H - 6.0));
                    if ui.add(widget).clicked() {
                        *value = *option;
                    }
                }
            });
        });
}

pub fn toggle(ui: &mut Ui, on: bool, label: &str) -> bool {
    let text = RichText::new(label)
        .font(theme::bold(14.0))
        .color(if on { ACCENT } else { MUTED });
    let width = (label.chars().count() as f32 * 15.0 + 28.0).max(148.0);
    let widget = egui::Button::new(text)
        .fill(if on { RAISED } else { SUNKEN })
        .stroke(Stroke::new(1.0, if on { LINE } else { LINE_SOFT }))
        .corner_radius(R_CTRL)
        .min_size(Vec2::new(width, CTRL_H));
    let hint = if on {
        "本窗口的按键正计入数字和导出。再点一次就取消。"
    } else {
        "本窗口的按键不计入数字和导出。再点一次就打开。"
    };
    ui.add(widget).on_hover_text(hint).clicked()
}

pub fn card() -> Frame {
    Frame::NONE
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, LINE_SOFT))
        .corner_radius(R_CARD)
        .inner_margin(PAD)
}

pub fn pill(ui: &mut Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        theme::bold(CAPTION),
        color,
    );
    let size = Vec2::new(galley.size().x + 30.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 12.0, color.gamma_multiply(0.16));
    painter.circle_filled(Pos2::new(rect.left() + 12.0, rect.center().y), 3.5, color);
    painter.galley(
        Pos2::new(rect.left() + 21.0, rect.center().y - galley.size().y * 0.5),
        galley,
        color,
    );
}

pub fn logo(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, size * 0.26, ACCENT);
    painter.text(
        rect.center() + Vec2::new(0.0, size * 0.02),
        Align2::CENTER_CENTER,
        "K",
        theme::bold(size * 0.62),
        ACCENT_INK,
    );
}

pub fn caption(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).size(CAPTION).color(MUTED)
}

pub fn section(ui: &mut Ui, title: &str, note: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).font(theme::bold(15.0)).color(TEXT));
        if !note.is_empty() {
            ui.label(caption(note));
        }
    });
}
