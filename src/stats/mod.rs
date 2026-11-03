mod query;

use crate::pace::Pace;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Row {
    pub process: String,
    pub key: u16,
    pub count: u64,
}

#[derive(Clone)]
pub struct Snapshot {
    pub rows: Vec<Row>,
    pub keyboard: u64,
    pub mouse: u64,
    pub elapsed: Duration,
    pub running: bool,
    pub dropped: u64,
    pub recent: u64,
}

pub struct Dump {
    pub rows: Vec<Row>,
    pub elapsed: Duration,
    pub running: bool,
    pub dropped: u64,
    pub count_self: bool,
    pub revision: u64,
}

enum Phase {
    Idle,
    Running { since: std::time::Instant, acc: Duration },
    Paused { acc: Duration },
}

struct Inner {
    session: u64,
    running: bool,
    count_self: bool,
    revision: u64,
    phase: Phase,
    self_name: Arc<str>,
    pace: Pace,
    counts: HashMap<Arc<str>, HashMap<u16, u64>>,
}

pub struct Stats {
    published: AtomicU64,
    dropped: AtomicU64,
    inner: Mutex<Inner>,
}

impl Stats {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            published: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            inner: Mutex::new(Inner {
                session: 1,
                running: false,
                count_self: false,
                revision: 0,
                phase: Phase::Idle,
                self_name: own_process_name(),
                pace: Pace::new(),
                counts: HashMap::new(),
            }),
        })
    }

    pub fn session(&self) -> u64 {
        self.published.load(Ordering::Acquire)
    }

    pub fn count_self(&self) -> bool {
        self.inner.lock().count_self
    }

    pub fn set_count_self(&self, count_self: bool) {
        let mut inner = self.inner.lock();
        if inner.count_self != count_self {
            inner.count_self = count_self;
            bump(&mut inner);
        }
    }

    pub fn note_drop(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn start(&self) {
        let mut inner = self.inner.lock();
        if inner.running {
            return;
        }
        let acc = match inner.phase {
            Phase::Paused { acc } => acc,
            _ => Duration::ZERO,
        };
        inner.phase = Phase::Running {
            since: std::time::Instant::now(),
            acc,
        };
        inner.running = true;
        self.published.store(inner.session, Ordering::Release);
    }

    pub fn pause(&self) {
        self.published.store(0, Ordering::Release);
        let mut inner = self.inner.lock();
        if let Phase::Running { since, acc } = inner.phase {
            inner.phase = Phase::Paused {
                acc: acc + since.elapsed(),
            };
        }
        inner.running = false;
        bump(&mut inner);
    }

    pub fn reset(&self) {
        self.published.store(0, Ordering::Release);
        let mut inner = self.inner.lock();
        inner.session = inner.session.wrapping_add(1).max(1);
        inner.running = false;
        inner.phase = Phase::Idle;
        inner.counts.clear();
        inner.pace.clear();
        bump(&mut inner);
        self.dropped.store(0, Ordering::Relaxed);
    }

    pub fn restore(&self, elapsed: Duration, dropped: u64, rows: Vec<Row>) {
        let mut inner = self.inner.lock();
        inner.counts.clear();
        for row in rows {
            let keys = inner.counts.entry(Arc::from(row.process)).or_default();
            let slot = keys.entry(row.key).or_insert(0);
            *slot = slot.saturating_add(row.count);
        }
        inner.phase = Phase::Paused { acc: elapsed };
        inner.running = false;
        inner.pace.clear();
        bump(&mut inner);
        self.dropped.store(dropped, Ordering::Relaxed);
        self.published.store(0, Ordering::Release);
    }

    pub fn ingest(&self, session: u64, name: Arc<str>, key: u16) {
        let mut inner = self.inner.lock();
        if session != inner.session {
            return;
        }
        let own = same_process(&name, &inner.self_name);
        let row = inner.counts.entry(name).or_default();
        let slot = row.entry(key).or_insert(0);
        *slot = slot.saturating_add(1);
        inner.pace.hit(own);
        bump(&mut inner);
    }
}

fn bump(inner: &mut Inner) {
    inner.revision = inner.revision.saturating_add(1);
}

fn same_process(name: &str, own: &str) -> bool {
    name.eq_ignore_ascii_case(own)
}

fn own_process_name() -> Arc<str> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.file_name().map(|name| Arc::from(name.to_string_lossy().as_ref())))
        .unwrap_or_else(|| Arc::from("keystats.exe"))
}
