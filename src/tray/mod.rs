mod host;
mod icon;

use crate::live;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU8, Ordering};
use std::thread::{self, JoinHandle};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

const SHOW_BIT: u8 = 1;
const QUIT_BIT: u8 = 2;
const PENDING: u8 = 0;
const READY: u8 = 1;
const FAILED: u8 = 2;

static EVENTS: AtomicU8 = AtomicU8::new(0);
static HOST: AtomicIsize = AtomicIsize::new(0);
static ICON: AtomicU8 = AtomicU8::new(PENDING);
static STOP: AtomicBool = AtomicBool::new(false);

pub struct Tray {
    thread: Option<JoinHandle<()>>,
}

impl Tray {
    pub fn start() -> Self {
        let thread = thread::Builder::new()
            .name("keystats-tray".into())
            .spawn(host::run)
            .ok();
        Self { thread }
    }

    pub fn poll(&self) -> (bool, bool) {
        let bits = EVENTS.swap(0, Ordering::AcqRel);
        (bits & SHOW_BIT != 0, bits & QUIT_BIT != 0)
    }

    pub fn icon_failed(&self) -> bool {
        ICON.load(Ordering::Acquire) == FAILED
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        STOP.store(true, Ordering::Release);
        post_close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(super) fn signal(bit: u8) {
    EVENTS.fetch_or(bit, Ordering::Release);
    if bit == SHOW_BIT {
        // 藏起窗口后重绘不会来，先用系统调用把它显示出来。
        crate::instance::reveal_window();
    }
    live::nudge();
}

pub(super) fn remember(hwnd: HWND) {
    HOST.store(hwnd.0 as isize, Ordering::Release);
}

pub(super) fn clear_host() {
    HOST.store(0, Ordering::Release);
}

pub(super) fn stopping() -> bool {
    STOP.load(Ordering::Acquire)
}

pub(super) fn mark_icon(ok: bool) {
    ICON.store(if ok { READY } else { FAILED }, Ordering::Release);
}

pub(super) const SHOW: u8 = SHOW_BIT;
pub(super) const QUIT: u8 = QUIT_BIT;

fn post_close() {
    let raw = HOST.load(Ordering::Acquire);
    if raw == 0 {
        return;
    }
    let hwnd = HWND(raw as *mut core::ffi::c_void);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
}
