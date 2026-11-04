use crate::keys::{is_mouse, MOUSE_LEFT, NUMPAD_ENTER};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

pub(crate) const FADE_MS: u64 = 240;
const MODS: [u16; 8] = [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0x5B, 0x5C];

struct Slot {
    down: AtomicBool,
    up_at: AtomicU64,
}

const SLOT: Slot = Slot {
    down: AtomicBool::new(false),
    up_at: AtomicU64::new(0),
};

static KEYS: [Slot; 257] = [SLOT; 257];
static POINTER: [Slot; 9] = [SLOT; 9];
static LATCHED: AtomicU16 = AtomicU16::new(0);
static WATCH: AtomicBool = AtomicBool::new(false);
static ORIGIN: OnceLock<Instant> = OnceLock::new();
static CTX: Mutex<Option<egui::Context>> = Mutex::new(None);

pub fn sync(ctx: &egui::Context, heat: bool) {
    {
        let mut slot = CTX.lock();
        if slot.is_none() {
            *slot = Some(ctx.clone());
        }
    }
    WATCH.store(heat, Ordering::Relaxed);
    if heat {
        refresh_latched();
    }
    let wait = if heat && fading() { 16 } else { 300 };
    ctx.request_repaint_after(Duration::from_millis(wait));
}

pub fn press(key: u16) {
    if let Some(slot) = slot(key) {
        slot.down.store(true, Ordering::Relaxed);
        wake();
    }
}

pub fn release(key: u16) {
    if let Some(slot) = slot(key) {
        slot.down.store(false, Ordering::Relaxed);
        slot.up_at.store(now_ms(), Ordering::Relaxed);
        wake();
    }
}

/// 滚轮没有按住，只亮一下再渐暗。
pub fn blip(key: u16) {
    release(key);
}

pub fn clear() {
    for slot in KEYS.iter().chain(POINTER.iter()) {
        slot.down.store(false, Ordering::Relaxed);
    }
    LATCHED.store(0, Ordering::Relaxed);
}

/// 0 是不亮，1 是常亮。
pub fn glow(key: u16) -> f32 {
    let Some(slot) = slot(key) else {
        return 0.0;
    };
    let up_at = slot.up_at.load(Ordering::Relaxed);
    let since = (up_at != 0).then(|| now_ms().saturating_sub(up_at));
    classify(slot.down.load(Ordering::Relaxed), latched(key), since)
}

pub fn classify(held: bool, sticky: bool, since_up_ms: Option<u64>) -> f32 {
    if held || sticky {
        return 1.0;
    }
    match since_up_ms {
        Some(age) if age < FADE_MS => {
            let t = 1.0 - age as f32 / FADE_MS as f32;
            t * t
        }
        _ => 0.0,
    }
}

fn refresh_latched() {
    let mut bits = 0u16;
    for (index, vk) in MODS.iter().enumerate() {
        let ours = KEYS[*vk as usize].down.load(Ordering::Relaxed);
        if !ours && os_down(*vk) {
            bits |= 1 << index;
        }
    }
    LATCHED.store(bits, Ordering::Relaxed);
}

fn os_down(vk: u16) -> bool {
    (unsafe { GetAsyncKeyState(i32::from(vk)) }) < 0
}

fn latched(key: u16) -> bool {
    let bits = LATCHED.load(Ordering::Relaxed);
    MODS.iter()
        .position(|vk| *vk == key)
        .is_some_and(|index| bits & (1 << index) != 0)
}

fn fading() -> bool {
    let now = now_ms();
    KEYS.iter().chain(POINTER.iter()).any(|slot| {
        let up_at = slot.up_at.load(Ordering::Relaxed);
        up_at != 0 && now.saturating_sub(up_at) < FADE_MS && !slot.down.load(Ordering::Relaxed)
    })
}

fn slot(key: u16) -> Option<&'static Slot> {
    if key < 256 {
        Some(&KEYS[key as usize])
    } else if key == NUMPAD_ENTER {
        Some(&KEYS[256])
    } else if is_mouse(key) {
        Some(&POINTER[(key - MOUSE_LEFT) as usize])
    } else {
        None
    }
}

/// 从 1 开始，0 留给「从没松开过」。
fn now_ms() -> u64 {
    ORIGIN.get_or_init(Instant::now).elapsed().as_millis() as u64 + 1
}

fn wake() {
    if !WATCH.load(Ordering::Relaxed) {
        return;
    }
    if let Some(ctx) = CTX.lock().as_ref() {
        ctx.request_repaint();
    }
}
