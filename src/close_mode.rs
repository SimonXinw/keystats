use std::fs;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseMode {
    Tray,
    Quit,
}

impl CloseMode {
    pub fn load() -> Self {
        let Some(path) = location() else {
            return CloseMode::Tray;
        };
        let text = fs::read_to_string(&path)
            .or_else(|_| fs::read_to_string(path.with_extension("tmp")))
            .unwrap_or_default();
        parse(&text)
    }

    pub fn save(self) {
        let Some(path) = location() else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let tmp = path.with_extension("tmp");
        if fs::write(&tmp, token(self)).is_ok() {
            let _ = fs::remove_file(&path);
            let _ = fs::rename(&tmp, &path);
        }
    }
}

pub fn parse(text: &str) -> CloseMode {
    match text.trim() {
        "quit" => CloseMode::Quit,
        _ => CloseMode::Tray,
    }
}

fn token(mode: CloseMode) -> &'static str {
    match mode {
        CloseMode::Tray => "tray\n",
        CloseMode::Quit => "quit\n",
    }
}

fn location() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("keystats\\close.txt"))
}

#[cfg(test)]
fn write_at(path: &Path, mode: CloseMode) {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).unwrap();
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, token(mode)).unwrap();
    let _ = fs::remove_file(path);
    fs::rename(&tmp, path).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_text_stays_on_tray() {
        assert_eq!(parse(""), CloseMode::Tray);
        assert_eq!(parse("tray"), CloseMode::Tray);
        assert_eq!(parse("  quit \n"), CloseMode::Quit);
        assert_eq!(parse("QUIT"), CloseMode::Tray);
        assert_eq!(parse("nope"), CloseMode::Tray);
    }

    #[test]
    fn file_round_trip() {
        let dir = std::env::temp_dir().join(format!("keystats-close-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("close.txt");
        write_at(&path, CloseMode::Quit);
        let saved = parse(&fs::read_to_string(&path).unwrap());
        assert_eq!(saved, CloseMode::Quit);
        write_at(&path, CloseMode::Tray);
        assert_eq!(parse(&fs::read_to_string(&path).unwrap()), CloseMode::Tray);
        let _ = fs::remove_dir_all(&dir);
    }
}
