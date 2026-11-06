use super::icon::{self, MSG_ICON};
use super::{QUIT, SHOW};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_CLASS_ALREADY_EXISTS, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, PostMessageW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, TrackPopupMenu, TranslateMessage, MF_SEPARATOR, MF_STRING, MSG,
    TPM_NONOTIFY, TPM_RETURNCMD, WM_CLOSE, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NULL,
    WM_RBUTTONUP, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};

static MENU: AtomicBool = AtomicBool::new(false);
static CLOSED: AtomicBool = AtomicBool::new(false);

pub(super) fn run() {
    let Some(hwnd) = (unsafe { create_host() }) else {
        super::mark_icon(false);
        return;
    };
    super::remember(hwnd);
    super::mark_icon(add_with_retry(hwnd));
    if super::stopping() {
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
    unsafe { pump() };
    super::clear_host();
}

fn add_with_retry(hwnd: HWND) -> bool {
    for _ in 0..5 {
        if super::stopping() {
            return false;
        }
        if unsafe { icon::add(hwnd) } {
            return true;
        }
        thread::sleep(Duration::from_millis(200));
    }
    false
}

unsafe fn pump() {
    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

unsafe fn create_host() -> Option<HWND> {
    let module = unsafe { GetModuleHandleW(None) }.ok()?;
    let instance = HINSTANCE::from(module);
    let class_name = w!("keystats.tray");
    let mut klass: WNDCLASSW = unsafe { std::mem::zeroed() };
    klass.lpfnWndProc = Some(wndproc);
    klass.hInstance = instance;
    klass.lpszClassName = class_name;
    let registered = unsafe { RegisterClassW(&klass) };
    if registered == 0 && unsafe { GetLastError() } != ERROR_CLASS_ALREADY_EXISTS {
        return None;
    }
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            class_name,
            WS_POPUP,
            -32000,
            -32000,
            1,
            1,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .ok()?;
    // 不显示这扇空窗口，避免任务栏和 Alt+Tab 里多出一个托盘宿主。
    Some(hwnd)
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        MSG_ICON => {
            on_icon(hwnd, lparam.0 as u32);
            LRESULT(0)
        }
        WM_CLOSE => {
            close_host(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn close_host(hwnd: HWND) {
    if CLOSED.swap(true, Ordering::AcqRel) {
        return;
    }
    unsafe {
        icon::remove(hwnd);
        let _ = DestroyWindow(hwnd);
    }
}

fn on_icon(hwnd: HWND, mouse: u32) {
    match mouse {
        WM_LBUTTONUP | WM_LBUTTONDBLCLK => super::signal(SHOW),
        WM_RBUTTONUP => popup(hwnd),
        _ => {}
    }
}

fn popup(hwnd: HWND) {
    if MENU.swap(true, Ordering::AcqRel) {
        return;
    }
    unsafe { show_menu(hwnd) };
    MENU.store(false, Ordering::Release);
}

unsafe fn show_menu(hwnd: HWND) {
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
        return;
    };
    unsafe {
        let _ = AppendMenuW(menu, MF_STRING, 1, w!("打开窗口"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, 2, w!("退出"));
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);
        let picked = TrackPopupMenu(
            menu,
            TPM_NONOTIFY | TPM_RETURNCMD,
            point.x,
            point.y,
            Some(0),
            hwnd,
            None,
        );
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        match picked.0 as usize {
            1 => super::signal(SHOW),
            2 => super::signal(QUIT),
            _ => {}
        }
    }
}
