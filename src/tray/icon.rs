use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{LoadIconW, HICON, IDI_APPLICATION};

pub(super) const MSG_ICON: u32 = 0x8000 + 31;

const ICON_ID: u32 = 1;
const TIP: &str = "按键统计。单击打开，右键退出。";

pub(super) unsafe fn add(hwnd: HWND) -> bool {
    let mut data = blank(hwnd);
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = MSG_ICON;
    data.hIcon = load_icon();
    data.szTip = tip_units();
    unsafe { Shell_NotifyIconW(NIM_ADD, &data).0 != 0 }
}

pub(super) unsafe fn remove(hwnd: HWND) {
    let data = blank(hwnd);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

fn blank(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = ICON_ID;
    data
}

fn load_icon() -> HICON {
    let instance = unsafe { GetModuleHandleW(None) }.ok().map(HINSTANCE::from);
    unsafe { LoadIconW(instance, PCWSTR(1u32 as _)) }
        .or_else(|_| unsafe { LoadIconW(None, IDI_APPLICATION) })
        .unwrap_or_default()
}

fn tip_units() -> [u16; 128] {
    let mut tip = [0u16; 128];
    for (slot, unit) in tip.iter_mut().zip(TIP.encode_utf16()) {
        *slot = unit;
    }
    tip
}
