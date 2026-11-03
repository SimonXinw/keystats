use std::thread;
use std::time::Duration;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

pub const TITLE: &str = "按键统计-keystats";

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
    true
}

fn focus_running() {
    let title: windows::core::HSTRING = TITLE.into();
    for _ in 0..20 {
        if let Ok(hwnd) = unsafe { FindWindowW(PCWSTR::null(), &title) } {
            unsafe {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
            }
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}
