use crate::keys::{self, category, is_mouse};
use crate::stats::Row;
use crate::ui_grid::{self, Col, Kind};
use egui::Ui;
use std::collections::BTreeMap;

const DETAIL: [Col; 4] = [
    Col { title: "进程", share: 0.36, right: false, kind: Kind::Name },
    Col { title: "按键", share: 0.22, right: false, kind: Kind::Strong },
    Col { title: "类别", share: 0.12, right: false, kind: Kind::Tag },
    Col { title: "次数", share: 0.30, right: true, kind: Kind::Total },
];

const PROCESS: [Col; 4] = [
    Col { title: "进程", share: 0.34, right: false, kind: Kind::Strong },
    Col { title: "键盘", share: 0.18, right: true, kind: Kind::Name },
    Col { title: "鼠标", share: 0.18, right: true, kind: Kind::Name },
    Col { title: "合计", share: 0.30, right: true, kind: Kind::Total },
];

const KEYS: [Col; 4] = [
    Col { title: "按键", share: 0.28, right: false, kind: Kind::Strong },
    Col { title: "类别", share: 0.16, right: false, kind: Kind::Tag },
    Col { title: "占比", share: 0.16, right: true, kind: Kind::Name },
    Col { title: "次数", share: 0.40, right: true, kind: Kind::Total },
];

pub fn detail(ui: &mut Ui, rows: &[Row], needle: &str) {
    let list: Vec<(&Row, String)> = rows
        .iter()
        .map(|row| (row, keys::label(row.key)))
        .filter(|(row, name)| matches(needle, &row.process, Some(name)))
        .collect();
    let max = list.iter().map(|(row, _)| row.count).max().unwrap_or(0);
    ui_grid::table(ui, &DETAIL, list.len(), |i| {
        let (row, name) = &list[i];
        (
            [
                row.process.clone(),
                name.clone(),
                category(row.key).to_owned(),
                row.count.to_string(),
            ],
            row.count,
            max,
        )
    });
}

pub fn process(ui: &mut Ui, rows: &[Row], needle: &str) {
    let mut sums: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
    for row in rows {
        if !matches(needle, &row.process, None) {
            continue;
        }
        let slot = sums.entry(row.process.as_str()).or_default();
        if is_mouse(row.key) {
            slot.1 = slot.1.saturating_add(row.count);
        } else {
            slot.0 = slot.0.saturating_add(row.count);
        }
    }
    let mut list: Vec<_> = sums.into_iter().collect();
    list.sort_by(|a, b| (b.1 .0 + b.1 .1).cmp(&(a.1 .0 + a.1 .1)));
    let max = list.iter().map(|(_, (k, m))| k + m).max().unwrap_or(0);
    ui_grid::table(ui, &PROCESS, list.len(), |i| {
        let (name, (keyboard, mouse)) = list[i];
        let total = keyboard + mouse;
        (
            [
                name.to_owned(),
                keyboard.to_string(),
                mouse.to_string(),
                total.to_string(),
            ],
            total,
            max,
        )
    });
}

/// 同一个键在所有进程里的次数加在一起，按次数从高到低排。
pub fn keys(ui: &mut Ui, rows: &[Row], needle: &str) {
    let mut sums: BTreeMap<u16, u64> = BTreeMap::new();
    for row in rows {
        let name = keys::label(row.key);
        if !matches(needle, &row.process, Some(&name)) {
            continue;
        }
        let slot = sums.entry(row.key).or_default();
        *slot = slot.saturating_add(row.count);
    }
    let mut list: Vec<_> = sums.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let max = list.iter().map(|(_, n)| *n).max().unwrap_or(0);
    let grand = list.iter().fold(0u64, |sum, (_, n)| sum.saturating_add(*n));
    ui_grid::table(ui, &KEYS, list.len(), |i| {
        let (key, count) = list[i];
        let share = if grand == 0 {
            "—".to_owned()
        } else {
            format!("{:.1}%", count as f64 * 100.0 / grand as f64)
        };
        (
            [keys::label(key), category(key).to_owned(), share, count.to_string()],
            count,
            max,
        )
    });
}

fn matches(needle: &str, process: &str, key_name: Option<&str>) -> bool {
    if needle.is_empty() {
        return true;
    }
    process.to_lowercase().contains(needle)
        || key_name.is_some_and(|name| name.to_lowercase().contains(needle))
}
