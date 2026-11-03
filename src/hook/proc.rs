use super::{InputEvent, SENDER, STATS};
use crate::keys::{
    self, normalize_vk, MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, MOUSE_X1, MOUSE_X2, WHEEL_DOWN,
    WHEEL_LEFT, WHEEL_RIGHT, WHEEL_UP,
};
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetWindowThreadProcessId, KBDLLHOOKSTRUCT, LLKHF_UP,
    MSLLHOOKSTRUCT, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_RBUTTONDOWN,
    WM_XBUTTONDOWN,
};

static DOWN: [AtomicBool; 256] = [const { AtomicBool::new(false) }; 256];
static NUMPAD_ENTER_DOWN: AtomicBool = AtomicBool::new(false);

pub fn clear_keyboard_state() {
    for key in &DOWN {
        key.store(false, Ordering::Relaxed);
    }
    NUMPAD_ENTER_DOWN.store(false, Ordering::Relaxed);
}

pub unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let pending = if code >= 0 && lparam.0 != 0 {
        take_keyboard(lparam).map(|key| (foreground_pid(), key))
    } else {
        None
    };
    let result = unsafe { CallNextHookEx(None, code, wparam, lparam) };
    if let Some((pid, key)) = pending {
        emit(pid, key);
    }
    result
}

pub unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let pending = if code >= 0 && lparam.0 != 0 {
        take_mouse(wparam, lparam).map(|key| (foreground_pid(), key))
    } else {
        None
    };
    let result = unsafe { CallNextHookEx(None, code, wparam, lparam) };
    if let Some((pid, key)) = pending {
        emit(pid, key);
    }
    result
}

fn take_keyboard(lparam: LPARAM) -> Option<u16> {
    let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let key = normalize_vk(info.vkCode, info.scanCode, info.flags.0);
    if info.flags.contains(LLKHF_UP) {
        mark_up(key);
        None
    } else if mark_down(key) {
        Some(key)
    } else {
        None
    }
}

fn take_mouse(wparam: WPARAM, lparam: LPARAM) -> Option<u16> {
    match wparam.0 as u32 {
        WM_LBUTTONDOWN => Some(MOUSE_LEFT),
        WM_RBUTTONDOWN => Some(MOUSE_RIGHT),
        WM_MBUTTONDOWN => Some(MOUSE_MIDDLE),
        WM_XBUTTONDOWN => Some(x_button(lparam)),
        WM_MOUSEWHEEL => wheel_key(lparam, true),
        WM_MOUSEHWHEEL => wheel_key(lparam, false),
        _ => None,
    }
}

fn x_button(lparam: LPARAM) -> u16 {
    let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    if ((info.mouseData >> 16) & 0xffff) == 2 {
        MOUSE_X2
    } else {
        MOUSE_X1
    }
}

fn wheel_key(lparam: LPARAM, vertical: bool) -> Option<u16> {
    let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    let delta = ((info.mouseData >> 16) & 0xffff) as u16 as i16;
    if delta == 0 {
        return None;
    }
    Some(if vertical {
        if delta > 0 {
            WHEEL_UP
        } else {
            WHEEL_DOWN
        }
    } else if delta > 0 {
        WHEEL_RIGHT
    } else {
        WHEEL_LEFT
    })
}

fn mark_down(key: u16) -> bool {
    if key == keys::NUMPAD_ENTER {
        return !NUMPAD_ENTER_DOWN.swap(true, Ordering::Relaxed);
    }
    if key > 255 {
        return true;
    }
    !DOWN[key as usize].swap(true, Ordering::Relaxed)
}

fn mark_up(key: u16) {
    if key == keys::NUMPAD_ENTER {
        NUMPAD_ENTER_DOWN.store(false, Ordering::Relaxed);
        return;
    }
    if key <= 255 {
        DOWN[key as usize].store(false, Ordering::Relaxed);
    }
}

fn emit(pid: u32, key: u16) {
    let stats = STATS.lock().clone();
    let Some(stats) = stats else {
        return;
    };
    let session = stats.session();
    if session == 0 {
        return;
    }
    let event = InputEvent { session, pid, key };
    let sender = SENDER.lock();
    let Some(sender) = sender.as_ref() else {
        return;
    };
    if sender.try_send(event).is_err() {
        stats.note_drop();
    }
}

fn foreground_pid() -> u32 {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return 0;
    }
    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    pid
}
