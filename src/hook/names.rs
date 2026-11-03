use crate::process;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Slot {
    name: Arc<str>,
    at: Instant,
}

pub struct NameCache {
    slots: HashMap<u32, Slot>,
}

impl NameCache {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    pub fn lookup(&mut self, pid: u32) -> Arc<str> {
        lookup(&mut self.slots, pid)
    }
}

fn lookup(cache: &mut HashMap<u32, Slot>, pid: u32) -> Arc<str> {
    if pid == 0 {
        return Arc::from("未知进程");
    }
    if let Some(slot) = cache.get(&pid) {
        if slot.at.elapsed() < Duration::from_secs(3) {
            return Arc::clone(&slot.name);
        }
    }
    let fresh = process::process_name(pid);
    if let Some(slot) = cache.get_mut(&pid) {
        if slot.name.as_ref() != fresh {
            slot.name = Arc::from(fresh);
        }
        slot.at = Instant::now();
        return Arc::clone(&slot.name);
    }
    let name: Arc<str> = Arc::from(fresh);
    remember(cache, pid, Arc::clone(&name));
    name
}

fn remember(cache: &mut HashMap<u32, Slot>, pid: u32, name: Arc<str>) {
    if cache.len() >= 512 {
        cache.clear();
    }
    cache.insert(
        pid,
        Slot {
            name,
            at: Instant::now(),
        },
    );
}
