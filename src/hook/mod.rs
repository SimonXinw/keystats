mod names;
mod proc;

use crate::stats::Stats;
use parking_lot::Mutex;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PeekMessageW, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, HHOOK, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_QUIT,
};

pub(super) struct InputEvent {
    session: u64,
    pid: u32,
    key: u16,
}

pub(super) static STATS: Mutex<Option<Arc<Stats>>> = Mutex::new(None);
pub(super) static SENDER: Mutex<Option<SyncSender<InputEvent>>> = Mutex::new(None);

pub struct Hooks {
    thread_id: Option<u32>,
    sender: Option<SyncSender<InputEvent>>,
    hook_thread: Option<JoinHandle<()>>,
    worker: Option<JoinHandle<()>>,
}

impl Hooks {
    pub fn start(stats: Arc<Stats>) -> Result<Self, String> {
        let (tx, rx) = sync_channel(2048);
        *SENDER.lock() = Some(tx.clone());
        *STATS.lock() = Some(Arc::clone(&stats));
        let worker = spawn_worker(rx, Arc::clone(&stats))?;
        let (boot_tx, boot_rx) = std::sync::mpsc::channel();
        let hook_thread = match spawn_hook(boot_tx) {
            Ok(thread) => thread,
            Err(err) => {
                release(Some(tx), None, Some(worker));
                return Err(err);
            }
        };
        match boot_rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(id)) => Ok(Self {
                thread_id: Some(id),
                sender: Some(tx),
                hook_thread: Some(hook_thread),
                worker: Some(worker),
            }),
            Ok(Err(err)) => {
                release(Some(tx), Some(hook_thread), Some(worker));
                Err(err)
            }
            Err(_) => {
                release(Some(tx), Some(hook_thread), Some(worker));
                Err("安装键盘鼠标监听超时".into())
            }
        }
    }

    pub fn clear_keyboard_state(&self) {
        proc::clear_keyboard_state();
    }

    fn shutdown(&mut self) {
        if let Some(id) = self.thread_id.take() {
            unsafe {
                let _ = PostThreadMessageW(id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
        release(
            self.sender.take(),
            self.hook_thread.take(),
            self.worker.take(),
        );
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn spawn_worker(rx: Receiver<InputEvent>, stats: Arc<Stats>) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("keystats-agg".into())
        .spawn(move || aggregate(rx, stats))
        .map_err(|err| err.to_string())
}

fn spawn_hook(
    boot_tx: std::sync::mpsc::Sender<Result<u32, String>>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("keystats-hook".into())
        .spawn(move || hook_main(boot_tx))
        .map_err(|err| err.to_string())
}

fn release(
    sender: Option<SyncSender<InputEvent>>,
    hook_thread: Option<JoinHandle<()>>,
    worker: Option<JoinHandle<()>>,
) {
    if let Some(thread) = hook_thread {
        let _ = thread.join();
    }
    *SENDER.lock() = None;
    drop(sender);
    if let Some(thread) = worker {
        let _ = thread.join();
    }
    *STATS.lock() = None;
}

fn aggregate(rx: Receiver<InputEvent>, stats: Arc<Stats>) {
    unsafe {
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
    }
    let mut cache = names::NameCache::new();
    while let Ok(event) = rx.recv() {
        let name = cache.lookup(event.pid);
        stats.ingest(event.session, name, event.key);
    }
}

fn hook_main(boot_tx: std::sync::mpsc::Sender<Result<u32, String>>) {
    let (keyboard, mouse) = match unsafe { install_hooks() } {
        Ok(hooks) => hooks,
        Err(err) => {
            let _ = boot_tx.send(Err(err));
            return;
        }
    };
    let mut message = MSG::default();
    unsafe {
        let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE);
        let _ = boot_tx.send(Ok(GetCurrentThreadId()));
        while GetMessageW(&mut message, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        let _ = UnhookWindowsHookEx(keyboard);
        let _ = UnhookWindowsHookEx(mouse);
    }
}

unsafe fn install_hooks() -> Result<(HHOOK, HHOOK), String> {
    let module = unsafe { GetModuleHandleW(None) }.map_err(|err| err.to_string())?;
    let instance = HINSTANCE::from(module);
    let keyboard =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(proc::keyboard_proc), Some(instance), 0) }
            .map_err(|err| err.to_string())?;
    match unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(proc::mouse_proc), Some(instance), 0) } {
        Ok(mouse) => Ok((keyboard, mouse)),
        Err(err) => {
            unsafe {
                let _ = UnhookWindowsHookEx(keyboard);
            }
            Err(err.to_string())
        }
    }
}
