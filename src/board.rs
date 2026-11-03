use crate::keys::NUMPAD_ENTER;

const GAP: u16 = 0xFFFE;
const STEP: f32 = 1.15;
const TALL: f32 = STEP + 1.0;

pub struct Spot {
    pub key: u16,
    pub label: &'static str,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn keyboard() -> Vec<Spot> {
    let mut out = Vec::with_capacity(108);
    let y1 = 1.35;
    let y2 = y1 + STEP;
    let y3 = y2 + STEP;
    let y4 = y3 + STEP;
    let y5 = y4 + STEP;
    lay(&mut out, 0.0, &FN_ROW);
    lay(&mut out, y1, &NUM_ROW);
    lay(&mut out, y2, &Q_ROW);
    lay(&mut out, y3, &A_ROW);
    lay(&mut out, y4, &Z_ROW);
    lay(&mut out, y5, &SPACE_ROW);
    out.push(spot(0x6B, "+", 22.0, y2, 1.0, TALL));
    out.push(spot(NUMPAD_ENTER, "Enter", 22.0, y4, 1.0, TALL));
    out
}

pub fn bounds(spots: &[Spot]) -> (f32, f32) {
    let width = spots.iter().map(|s| s.x + s.w).fold(0.0, f32::max);
    let height = spots.iter().map(|s| s.y + s.h).fold(0.0, f32::max);
    (width, height)
}

fn spot(key: u16, label: &'static str, x: f32, y: f32, w: f32, h: f32) -> Spot {
    Spot {
        key,
        label,
        x,
        y,
        w,
        h,
    }
}

fn lay(out: &mut Vec<Spot>, y: f32, items: &'static [(&'static str, u16, f32)]) {
    let mut x = 0.0;
    for (label, key, w) in items {
        if *key != GAP {
            out.push(spot(*key, label, x, y, *w, 1.0));
        }
        x += *w;
    }
}

const FN_ROW: &[(&str, u16, f32)] = &[
    ("Esc", 0x1B, 1.0),
    ("", GAP, 1.0),
    ("F1", 0x70, 1.0),
    ("F2", 0x71, 1.0),
    ("F3", 0x72, 1.0),
    ("F4", 0x73, 1.0),
    ("", GAP, 0.5),
    ("F5", 0x74, 1.0),
    ("F6", 0x75, 1.0),
    ("F7", 0x76, 1.0),
    ("F8", 0x77, 1.0),
    ("", GAP, 0.5),
    ("F9", 0x78, 1.0),
    ("F10", 0x79, 1.0),
    ("F11", 0x7A, 1.0),
    ("F12", 0x7B, 1.0),
    ("", GAP, 0.5),
    ("Prt", 0x2C, 1.0),
    ("Scr", 0x91, 1.0),
    ("Pse", 0x13, 1.0),
];

const NUM_ROW: &[(&str, u16, f32)] = &[
    ("`", 0xC0, 1.0),
    ("1", 0x31, 1.0),
    ("2", 0x32, 1.0),
    ("3", 0x33, 1.0),
    ("4", 0x34, 1.0),
    ("5", 0x35, 1.0),
    ("6", 0x36, 1.0),
    ("7", 0x37, 1.0),
    ("8", 0x38, 1.0),
    ("9", 0x39, 1.0),
    ("0", 0x30, 1.0),
    ("-", 0xBD, 1.0),
    ("=", 0xBB, 1.0),
    ("Back", 0x08, 2.0),
    ("", GAP, 0.5),
    ("Ins", 0x2D, 1.0),
    ("Home", 0x24, 1.0),
    ("PgUp", 0x21, 1.0),
    ("", GAP, 0.5),
    ("Num", 0x90, 1.0),
    ("/", 0x6F, 1.0),
    ("*", 0x6A, 1.0),
    ("-", 0x6D, 1.0),
];

const Q_ROW: &[(&str, u16, f32)] = &[
    ("Tab", 0x09, 1.5),
    ("Q", 0x51, 1.0),
    ("W", 0x57, 1.0),
    ("E", 0x45, 1.0),
    ("R", 0x52, 1.0),
    ("T", 0x54, 1.0),
    ("Y", 0x59, 1.0),
    ("U", 0x55, 1.0),
    ("I", 0x49, 1.0),
    ("O", 0x4F, 1.0),
    ("P", 0x50, 1.0),
    ("[", 0xDB, 1.0),
    ("]", 0xDD, 1.0),
    ("\\", 0xDC, 1.5),
    ("", GAP, 0.5),
    ("Del", 0x2E, 1.0),
    ("End", 0x23, 1.0),
    ("PgDn", 0x22, 1.0),
    ("", GAP, 0.5),
    ("7", 0x67, 1.0),
    ("8", 0x68, 1.0),
    ("9", 0x69, 1.0),
];

const A_ROW: &[(&str, u16, f32)] = &[
    ("Caps", 0x14, 1.75),
    ("A", 0x41, 1.0),
    ("S", 0x53, 1.0),
    ("D", 0x44, 1.0),
    ("F", 0x46, 1.0),
    ("G", 0x47, 1.0),
    ("H", 0x48, 1.0),
    ("J", 0x4A, 1.0),
    ("K", 0x4B, 1.0),
    ("L", 0x4C, 1.0),
    (";", 0xBA, 1.0),
    ("'", 0xDE, 1.0),
    ("Enter", 0x0D, 2.25),
    ("", GAP, 4.0),
    ("4", 0x64, 1.0),
    ("5", 0x65, 1.0),
    ("6", 0x66, 1.0),
];

const Z_ROW: &[(&str, u16, f32)] = &[
    ("Shift", 0xA0, 2.25),
    ("Z", 0x5A, 1.0),
    ("X", 0x58, 1.0),
    ("C", 0x43, 1.0),
    ("V", 0x56, 1.0),
    ("B", 0x42, 1.0),
    ("N", 0x4E, 1.0),
    ("M", 0x4D, 1.0),
    (",", 0xBC, 1.0),
    (".", 0xBE, 1.0),
    ("/", 0xBF, 1.0),
    ("Shift", 0xA1, 2.75),
    ("", GAP, 1.5),
    ("↑", 0x26, 1.0),
    ("", GAP, 1.5),
    ("1", 0x61, 1.0),
    ("2", 0x62, 1.0),
    ("3", 0x63, 1.0),
];

const SPACE_ROW: &[(&str, u16, f32)] = &[
    ("Ctrl", 0xA2, 1.25),
    ("Win", 0x5B, 1.25),
    ("Alt", 0xA4, 1.25),
    ("空格", 0x20, 6.25),
    ("Alt", 0xA5, 1.25),
    ("Win", 0x5C, 1.25),
    ("Menu", 0x5D, 1.25),
    ("Ctrl", 0xA3, 1.25),
    ("", GAP, 0.5),
    ("←", 0x25, 1.0),
    ("↓", 0x28, 1.0),
    ("→", 0x27, 1.0),
    ("", GAP, 0.5),
    ("0", 0x60, 2.0),
    (".", 0x6E, 1.0),
];
