use std::path::Path;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

pub fn process_name(pid: u32) -> String {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
    let Ok(handle) = handle else {
        return format!("进程 {pid}");
    };
    let mut buf = [0u16; MAX_PATH as usize];
    let mut len = buf.len() as u32;
    let queried = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
    };
    unsafe {
        let _ = CloseHandle(handle);
    }
    if queried.is_err() || len == 0 {
        return format!("进程 {pid}");
    }
    let full = String::from_utf16_lossy(&buf[..len as usize]);
    let full = full.trim_end_matches('\0');
    Path::new(&full)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("未知进程")
        .to_string()
}
