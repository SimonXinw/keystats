use crate::keys::{self, category};
use crate::stats::Row;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn write_csv(path: &Path, rows: &[Row]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut out = BufWriter::new(file);
    out.write_all(&[0xEF, 0xBB, 0xBF])?;
    writeln!(out, "进程,按键,类别,次数")?;
    for row in rows {
        writeln!(
            out,
            "{},{},{},{}",
            csv_cell(&row.process),
            csv_cell(&keys::label(row.key)),
            category(row.key),
            row.count
        )?;
    }
    out.flush()
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
