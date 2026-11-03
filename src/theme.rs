use egui::{Color32, FontData, FontDefinitions, FontFamily, Shadow, Stroke, Visuals};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(13, 16, 22);
pub const SURFACE: Color32 = Color32::from_rgb(20, 24, 33);
pub const RAISED: Color32 = Color32::from_rgb(30, 36, 48);
pub const HOVER: Color32 = Color32::from_rgb(40, 48, 64);
pub const SUNKEN: Color32 = Color32::from_rgb(10, 12, 17);
pub const LINE: Color32 = Color32::from_rgb(44, 52, 68);
pub const LINE_SOFT: Color32 = Color32::from_rgb(31, 37, 49);

pub const TEXT: Color32 = Color32::from_rgb(232, 236, 243);
pub const MUTED: Color32 = Color32::from_rgb(142, 153, 172);
pub const FAINT: Color32 = Color32::from_rgb(96, 106, 124);

pub const ACCENT: Color32 = Color32::from_rgb(46, 196, 182);
pub const ACCENT_INK: Color32 = Color32::from_rgb(8, 34, 32);
pub const WARM: Color32 = Color32::from_rgb(242, 186, 92);
pub const OK: Color32 = Color32::from_rgb(84, 214, 150);
pub const DANGER: Color32 = Color32::from_rgb(236, 110, 98);
pub const DANGER_INK: Color32 = Color32::from_rgb(46, 12, 10);

pub const R_CARD: u8 = 12;
pub const R_CTRL: u8 = 8;
pub const GAP: f32 = 12.0;
pub const PAD: i8 = 16;
pub const CTRL_H: f32 = 34.0;

pub const CAPTION: f32 = 12.0;
pub const BODY: f32 = 14.0;
pub const TITLE: f32 = 20.0;
pub const NUMBER: f32 = 28.0;

pub fn install(ctx: &egui::Context) {
    load_cjk(ctx);
    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = SURFACE;
    visuals.window_stroke = Stroke::new(1.0, LINE);
    visuals.window_corner_radius = R_CARD.into();
    visuals.menu_corner_radius = R_CTRL.into();
    visuals.window_shadow = shadow(24, 110);
    visuals.popup_shadow = shadow(14, 90);
    visuals.extreme_bg_color = SUNKEN;
    visuals.faint_bg_color = SURFACE;
    visuals.code_bg_color = RAISED;
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = Color32::from_rgb(24, 86, 82);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.text_cursor.stroke = Stroke::new(2.0, ACCENT);

    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = SURFACE;
    w.noninteractive.weak_bg_fill = SURFACE;
    w.noninteractive.bg_stroke = Stroke::new(1.0, LINE_SOFT);
    w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    for (state, fill, line) in [
        (&mut w.inactive, RAISED, LINE),
        (&mut w.hovered, HOVER, Color32::from_rgb(70, 82, 102)),
        (&mut w.active, Color32::from_rgb(50, 60, 80), ACCENT),
        (&mut w.open, HOVER, LINE),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, line);
        state.fg_stroke = Stroke::new(1.0, TEXT);
        state.corner_radius = R_CTRL.into();
        state.expansion = 0.0;
    }
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, visuals);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 7.0);
    style.spacing.interact_size.y = CTRL_H;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.tooltip_width = 260.0;
    for (text, size) in [
        (egui::TextStyle::Small, CAPTION),
        (egui::TextStyle::Body, BODY),
        (egui::TextStyle::Button, BODY),
        (egui::TextStyle::Heading, TITLE),
    ] {
        if let Some(font) = style.text_styles.get_mut(&text) {
            font.size = size;
        }
    }
    ctx.set_style_of(egui::Theme::Dark, style);
}

fn shadow(blur: u8, alpha: u8) -> Shadow {
    Shadow {
        offset: [0, 8],
        blur,
        spread: 0,
        color: Color32::from_black_alpha(alpha),
    }
}

pub fn bold_family() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn bold(size: f32) -> egui::FontId {
    egui::FontId::new(size, bold_family())
}

fn load_cjk(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let regular = read_font(&["Deng.ttf", "msyh.ttc", "simhei.ttf"]);
    let heavy = read_font(&["Dengb.ttf", "msyhbd.ttc"]);
    if let Some(bytes) = regular {
        fonts
            .font_data
            .insert("ui".to_owned(), Arc::new(FontData::from_owned(bytes)));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, "ui".to_owned());
        }
    }
    let mut chain = Vec::new();
    if let Some(bytes) = heavy {
        fonts
            .font_data
            .insert("bold".to_owned(), Arc::new(FontData::from_owned(bytes)));
        chain.push("bold".to_owned());
    }
    chain.extend(
        fonts
            .families
            .get(&FontFamily::Proportional)
            .cloned()
            .unwrap_or_default(),
    );
    fonts.families.insert(bold_family(), chain);
    ctx.set_fonts(fonts);
}

fn read_font(names: &[&str]) -> Option<Vec<u8>> {
    names
        .iter()
        .find_map(|name| std::fs::read(format!(r"C:\Windows\Fonts\{name}")).ok())
}
