use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
};

pub const TITLE: &str = "按键统计-keystats";

static SHOW: AtomicBool = AtomicBool::new(false);

/// 已经有一份在跑时，把那一扇窗口带到前面，不再装第二套钩子。
pub fn primary() -> bool {
    let handle = unsafe { CreateMutexW(None, false, w!("Local\\keystats.single")) };
    let Ok(handle) = handle else {
        return true;
    };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        let _ = unsafe { CloseHandle(handle) };
        focus_running();
        return false;
    }
    // 句柄没有 Drop。留到进程退出，系统才释放这把锁。
    let _held = handle;
    arm_show();
    true
}

/// 缩在托盘里时，再启动一次会把窗口要回来。
pub fn take_show() -> bool {
    SHOW.swap(false, Ordering::AcqRel)
}

fn focus_running() {
    for _ in 0..20 {
        signal_show();
        if reveal_window() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// 窗口藏起来之后，界面循环可能不再绘制。这里直接让系统把窗口显示出来。
pub fn reveal_window() -> bool {
    let title: windows::core::HSTRING = TITLE.into();
    let Ok(hwnd) = (unsafe { FindWindowW(PCWSTR::null(), &title) }) else {
        return false;
    };
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
    true
}

fn arm_show() {
    let Ok(handle) = (unsafe { CreateEventW(None, false, false, w!("Local\\keystats.show")) })
    else {
        return;
    };
    let raw = handle.0 as isize;
    thread::spawn(move || {
        let handle = HANDLE(raw as *mut core::ffi::c_void);
        loop {
            if unsafe { WaitForSingleObject(handle, 0xFFFFFFFF) } != WAIT_OBJECT_0 {
                break;
            }
            SHOW.store(true, Ordering::Release);
            reveal_window();
            crate::live::nudge();
        }
    });
}

fn signal_show() {
    let Ok(handle) = (unsafe { OpenEventW(EVENT_MODIFY_STATE, false, w!("Local\\keystats.show")) })
    else {
        return;
    };
    let _ = unsafe { SetEvent(handle) };
    let _ = unsafe { CloseHandle(handle) };
}
