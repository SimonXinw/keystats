use super::{same_process, Dump, Phase, Row, Snapshot, Stats};
use crate::keys::is_mouse;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

impl Stats {
    pub fn snapshot(&self) -> Snapshot {
        let mut inner = self.inner.lock();
        let count_self = inner.count_self;
        let recent = inner.pace.recent(count_self);
        let mut rows = Vec::new();
        let mut keyboard = 0u64;
        let mut mouse = 0u64;
        for (process, keys) in &inner.counts {
            if !count_self && same_process(process, &inner.self_name) {
                continue;
            }
            for (key, count) in keys {
                if is_mouse(*key) {
                    mouse = mouse.saturating_add(*count);
                } else {
                    keyboard = keyboard.saturating_add(*count);
                }
                rows.push(row_of(process, *key, *count));
            }
        }
        sort_rows(&mut rows);
        Snapshot {
            rows,
            keyboard,
            mouse,
            elapsed: elapsed_of(&inner.phase),
            running: inner.running,
            dropped: self.dropped.load(Ordering::Relaxed),
            recent,
        }
    }

    pub fn dump(&self) -> Dump {
        let inner = self.inner.lock();
        let mut rows = Vec::new();
        for (process, keys) in &inner.counts {
            for (key, count) in keys {
                rows.push(row_of(process, *key, *count));
            }
        }
        sort_rows(&mut rows);
        Dump {
            rows,
            elapsed: elapsed_of(&inner.phase),
            running: inner.running,
            dropped: self.dropped.load(Ordering::Relaxed),
            count_self: inner.count_self,
            revision: inner.revision,
        }
    }
}

fn row_of(process: &Arc<str>, key: u16, count: u64) -> Row {
    Row {
        process: process.to_string(),
        key,
        count,
    }
}

fn sort_rows(rows: &mut [Row]) {
    rows.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.process.cmp(&b.process))
            .then_with(|| a.key.cmp(&b.key))
    });
}

fn elapsed_of(phase: &Phase) -> Duration {
    match *phase {
        Phase::Idle => Duration::ZERO,
        Phase::Paused { acc } => acc,
        Phase::Running { since, acc } => acc + since.elapsed(),
    }
}
