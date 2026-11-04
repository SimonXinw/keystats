use crate::theme::TEXT;
use egui::Color32;
use std::path::PathBuf;

pub const SLOTS: [(&str, &str); 4] = [
    ("cold", "次数少"),
    ("mid", "次数中"),
    ("hot", "次数多"),
    ("lamp", "按下"),
];

#[derive(Clone, Copy, PartialEq)]
pub struct Palette {
    pub cold: Color32,
    pub mid: Color32,
    pub hot: Color32,
    pub lamp: Color32,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            cold: Color32::from_rgb(28, 66, 86),
            mid: Color32::from_rgb(40, 182, 170),
            hot: Color32::from_rgb(244, 188, 92),
            lamp: Color32::from_rgb(242, 246, 255),
        }
    }
}

impl Palette {
    pub fn ramp(&self, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        if t < 0.6 {
            mix(self.cold, self.mid, t / 0.6)
        } else {
            mix(self.mid, self.hot, (t - 0.6) / 0.4)
        }
    }

    pub fn get(&self, index: usize) -> Color32 {
        match index {
            0 => self.cold,
            1 => self.mid,
            2 => self.hot,
            _ => self.lamp,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Color32 {
        match index {
            0 => &mut self.cold,
            1 => &mut self.mid,
            2 => &mut self.hot,
            _ => &mut self.lamp,
        }
    }

    pub fn load() -> Self {
        let mut palette = Self::default();
        let Some(text) = location().and_then(|path| std::fs::read_to_string(path).ok()) else {
            return palette;
        };
        for line in text.lines() {
            let Some((name, value)) = line.split_once('\t') else {
                continue;
            };
            let slot = SLOTS.iter().position(|(key, _)| *key == name);
            if let (Some(index), Some(color)) = (slot, parse_hex(value)) {
                *palette.get_mut(index) = color;
            }
        }
        palette
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = location().ok_or(std::io::Error::other("没有 LOCALAPPDATA"))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text: String = SLOTS
            .iter()
            .enumerate()
            .map(|(index, (key, _))| format!("{key}\t{}\n", to_hex(self.get(index))))
            .collect();
        std::fs::write(path, text)
    }
}

pub fn to_hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

/// 接受 `#RRGGBB`、`RRGGBB` 和简写 `#RGB`。
pub fn parse_hex(text: &str) -> Option<Color32> {
    let digits = text.trim().trim_start_matches('#');
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).ok();
    match digits.len() {
        6 => Some(Color32::from_rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?)),
        3 => {
            let short = |i: usize| channel(i..i + 1).map(|v| v * 17);
            Some(Color32::from_rgb(short(0)?, short(1)?, short(2)?))
        }
        _ => None,
    }
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}

/// 按底色亮度挑字色，换了配色也能看清。
pub fn ink_on(fill: Color32) -> Color32 {
    let luma = 0.2126 * fill.r() as f32 + 0.7152 * fill.g() as f32 + 0.0722 * fill.b() as f32;
    if luma > 140.0 {
        Color32::from_rgb(14, 22, 30)
    } else {
        TEXT
    }
}

fn location() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("keystats\\palette.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_and_rejects_junk() {
        let color = Color32::from_rgb(0x1C, 0x42, 0x56);
        assert_eq!(to_hex(color), "#1C4256");
        assert_eq!(parse_hex("#1c4256"), Some(color));
        assert_eq!(parse_hex(" 1C4256 "), Some(color));
        assert_eq!(parse_hex("#fff"), Some(Color32::WHITE));
        assert_eq!(parse_hex("#12345"), None);
        assert_eq!(parse_hex("#GG0000"), None);
        assert_eq!(parse_hex("#ＡＢＣ"), None);
    }
}
