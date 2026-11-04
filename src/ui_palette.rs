use crate::palette::{ink_on, parse_hex, to_hex, Palette, SLOTS};
use crate::theme::{self, CAPTION, FAINT, LINE, MUTED, R_CARD, SURFACE, TEXT};
use crate::ui_kit::{self, Tone};
use crate::ui_paint;
use egui::color_picker::{color_edit_button_srgba, Alpha};
use egui::{Align, Align2, Frame, Layout, RichText, Sense, Stroke, TextEdit, TextStyle, Ui, Vec2};

pub struct Colors {
    pub palette: Palette,
    saved: Palette,
    open: bool,
    hex: [String; 4],
}

impl Colors {
    pub fn load() -> Self {
        let palette = Palette::load();
        Self {
            palette,
            saved: palette,
            open: false,
            hex: std::array::from_fn(|i| to_hex(palette.get(i))),
        }
    }

    #[cfg(debug_assertions)]
    pub fn opened(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn button(&mut self, ui: &mut Ui) {
        let tone = if self.open { Tone::Primary } else { Tone::Plain };
        if ui_kit::button(ui, "配色", tone, true).clicked() {
            self.open = !self.open;
        }
    }

    pub fn window(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        let frame = Frame::window(&ctx.style())
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(R_CARD + 2)
            .inner_margin(18);
        let mut open = true;
        egui::Window::new(RichText::new("热力配色").font(theme::bold(16.0)).color(TEXT))
            .id(egui::Id::new("palette_editor"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .frame(frame)
            .pivot(Align2::RIGHT_TOP)
            .default_pos(ctx.screen_rect().right_top() + Vec2::new(-36.0, 140.0))
            .show(ctx, |ui| self.body(ui));
        self.open &= open;
    }

    /// 拖动取色时不写盘，松开鼠标再存。
    pub fn persist(&mut self, busy: bool) {
        if busy || self.palette == self.saved {
            return;
        }
        if self.palette.save().is_ok() {
            self.saved = self.palette;
        }
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.set_width(300.0);
        ui.label(ui_kit::caption("改动会立刻用在热力图上，并保存在本机。"));
        ui.add_space(10.0);
        preview(ui, &self.palette);
        ui.add_space(12.0);
        for index in 0..SLOTS.len() {
            if index == 3 {
                ui.add_space(6.0);
            }
            self.row(ui, index);
        }
        ui.add_space(14.0);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 36.0),
            Layout::right_to_left(Align::Center),
            |ui| {
                if ui_kit::button(ui, "完成", Tone::Primary, true).clicked() {
                    self.open = false;
                }
                if ui_kit::button(ui, "恢复默认", Tone::Plain, true).clicked() {
                    self.palette = Palette::default();
                }
            },
        );
    }

    fn row(&mut self, ui: &mut Ui, index: usize) {
        ui.horizontal(|ui| {
            let name = SLOTS[index].1;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(64.0, 24.0), Sense::hover());
            ui.painter().text(
                rect.left_center(),
                Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(14.0),
                TEXT,
            );
            color_edit_button_srgba(ui, self.palette.get_mut(index), Alpha::Opaque);
            let edit = TextEdit::singleline(&mut self.hex[index])
                .font(TextStyle::Monospace)
                .char_limit(7)
                .desired_width(96.0)
                .margin(egui::Margin::symmetric(10, 6));
            let response = ui.add(edit);
            if response.changed() {
                if let Some(color) = parse_hex(&self.hex[index]) {
                    *self.palette.get_mut(index) = color;
                }
            } else if !response.has_focus() {
                self.hex[index] = to_hex(self.palette.get(index));
            }
            let valid = parse_hex(&self.hex[index]).is_some();
            if response.has_focus() && !valid {
                ui.label(RichText::new("格式 #RRGGBB").size(CAPTION).color(FAINT));
            }
        });
    }
}

fn preview(ui: &mut Ui, palette: &Palette) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("少").size(CAPTION).color(MUTED));
        ui_paint::gradient(ui, palette, Vec2::new(176.0, 12.0));
        ui.label(RichText::new("多").size(CAPTION).color(MUTED));
        ui.add_space(6.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(46.0, 22.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect.expand(3.0), 8.0, palette.lamp.gamma_multiply(0.25));
        painter.rect_filled(rect, 6.0, palette.lamp);
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "按下",
            egui::FontId::proportional(CAPTION),
            ink_on(palette.lamp),
        );
    });
}
