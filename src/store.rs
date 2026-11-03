use crate::stats::{Dump, Row, Stats};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const NOTICE: &str = "已恢复上次的统计，点「继续」接着记。";

pub struct Keeper {
    armed: bool,
    saved_rev: u64,
    last_write: Instant,
}

impl Keeper {
    pub fn open(stats: &Stats) -> (Self, String) {
        if demo_mode() {
            return (Self::idle(0), String::new());
        }
        let Some(saved) = read() else {
            return (Self::idle(0), String::new());
        };
        stats.set_count_self(saved.count_self);
        let keep = !saved.rows.is_empty() || saved.elapsed >= Duration::from_secs(1);
        if keep {
            stats.restore(saved.elapsed, saved.dropped, saved.rows);
        }
        let rev = stats.dump().revision;
        let notice = if keep { NOTICE.into() } else { String::new() };
        (Self::idle(rev), notice)
    }

    pub fn tick(&mut self, stats: &Stats) {
        if !self.armed || self.last_write.elapsed() < Duration::from_secs(5) {
            return;
        }
        self.last_write = Instant::now();
        let dump = stats.dump();
        if !dump.running && dump.revision == self.saved_rev {
            return;
        }
        if write(&dump).is_ok() {
            self.saved_rev = dump.revision;
        }
    }

    pub fn flush(&mut self, stats: &Stats) {
        if !self.armed {
            return;
        }
        let dump = stats.dump();
        if write(&dump).is_ok() {
            self.saved_rev = dump.revision;
            self.last_write = Instant::now();
        }
    }

    fn idle(saved_rev: u64) -> Self {
        Self {
            armed: !demo_mode(),
            saved_rev,
            last_write: Instant::now(),
        }
    }
}

struct Saved {
    elapsed: Duration,
    dropped: u64,
    count_self: bool,
    rows: Vec<Row>,
}

fn location() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("keystats\\session.txt"))
}

fn read() -> Option<Saved> {
    let path = location()?;
    let text = fs::read_to_string(&path).or_else(|_| fs::read_to_string(path.with_extension("tmp")));
    parse(&text.ok()?)
}

fn write(dump: &Dump) -> std::io::Result<()> {
    let path = location().ok_or(std::io::Error::other("没有 LOCALAPPDATA"))?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    write_at(&tmp, dump)?;
    let _ = fs::remove_file(&path);
    fs::rename(&tmp, &path)
}

fn write_at(path: &Path, dump: &Dump) -> std::io::Result<()> {
    let mut out = BufWriter::new(File::create(path)?);
    writeln!(out, "1")?;
    writeln!(out, "elapsed_ms\t{}", dump.elapsed.as_millis())?;
    writeln!(out, "dropped\t{}", dump.dropped)?;
    writeln!(out, "count_self\t{}", u8::from(dump.count_self))?;
    writeln!(out, "---")?;
    for row in &dump.rows {
        if row.process.contains(['\t', '\n', '\r']) {
            continue;
        }
        writeln!(out, "{}\t{}\t{}", row.process, row.key, row.count)?;
    }
    out.flush()
}
fn parse(text: &str) -> Option<Saved> {
    let mut lines = text.lines();
    if lines.next()? != "1" {
        return None;
    }
    let mut saved = Saved {
        elapsed: Duration::ZERO,
        dropped: 0,
        count_self: false,
        rows: Vec::new(),
    };
    let mut body = false;
    for line in lines {
        if line == "---" {
            body = true;
            continue;
        }
        if body {
            if let Some(row) = parse_row(line) {
                saved.rows.push(row);
            }
        } else if let Some((key, value)) = line.split_once('\t') {
            apply_header(&mut saved, key, value)?;
        }
    }
    Some(saved)
}

fn apply_header(saved: &mut Saved, key: &str, value: &str) -> Option<()> {
    match key {
        "elapsed_ms" => saved.elapsed = Duration::from_millis(value.parse().ok()?),
        "dropped" => saved.dropped = value.parse().ok()?,
        "count_self" => saved.count_self = value == "1",
        _ => {}
    }
    Some(())
}
fn parse_row(line: &str) -> Option<Row> {
    let mut parts = line.split('\t');
    Some(Row {
        process: parts.next()?.to_string(),
        key: parts.next()?.parse().ok()?,
        count: parts.next()?.parse().ok()?,
    })
}
fn demo_mode() -> bool {
    #[cfg(debug_assertions)]
    {
        std::env::var_os("KEYSTATS_DEMO").is_some()
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_keeps_counts_elapsed_and_preference() {
        let dir = std::env::temp_dir().join(format!("keystats-store-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.txt");
        let dump = Dump {
            rows: vec![Row {
                process: "League of Legends.exe".into(),
                key: 0x51,
                count: 12,
            }],
            elapsed: Duration::from_millis(1500),
            running: false,
            dropped: 2,
            count_self: true,
            revision: 7,
        };
        write_at(&path, &dump).unwrap();
        let saved = parse(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved.elapsed, Duration::from_millis(1500));
        assert_eq!(saved.dropped, 2);
        assert!(saved.count_self);
        assert_eq!(saved.rows.len(), 1);
        assert_eq!(saved.rows[0].count, 12);
        assert!(parse("nope").is_none());
        let _ = fs::remove_dir_all(&dir);
    }
}
