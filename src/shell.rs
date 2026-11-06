use crate::close_mode::CloseMode;
use crate::instance;
use crate::tray::Tray;

const HIDDEN: &str = "已缩到托盘。单击图标打开，右键可以退出。";

pub struct Shell {
    mode: CloseMode,
    open: bool,
    quit: bool,
    tray: Tray,
}

impl Shell {
    pub fn start() -> Self {
        Self {
            mode: CloseMode::load(),
            open: demo_settings(),
            quit: false,
            tray: Tray::start(),
        }
    }

    pub fn settings_open(&self) -> bool {
        self.open
    }

    pub fn toggle_settings(&mut self) {
        self.open = !self.open;
    }

    /// 返回这一帧是不是刚缩进托盘，调用方趁这时把统计写盘。
    pub fn on_frame(&mut self, ctx: &egui::Context, notice: &mut String) -> bool {
        let (show, tray_quit) = self.tray.poll();
        self.quit |= tray_quit;
        if (show || instance::take_show()) && !self.quit {
            reveal(ctx);
        }
        let hid = if self.quit {
            false
        } else {
            self.on_close(ctx, notice)
        };
        if !self.quit {
            let before = self.mode;
            self.quit |= crate::ui_settings::show(ctx, &mut self.open, &mut self.mode);
            if self.mode != before {
                self.mode.save();
            }
        }
        if self.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        hid
    }

    fn on_close(&self, ctx: &egui::Context, notice: &mut String) -> bool {
        let requested = ctx.input(|input| input.viewport().close_requested());
        if !requested || self.mode != CloseMode::Tray || self.tray.icon_failed() || shot_exit() {
            return false;
        }
        // 系统已经发出关闭。取消后窗口还在，再把它从任务栏拿掉。
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        *notice = HIDDEN.into();
        true
    }
}

fn reveal(ctx: &egui::Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}

fn shot_exit() -> bool {
    std::env::var_os("KEYSTATS_SHOT").is_some()
}

fn demo_settings() -> bool {
    #[cfg(debug_assertions)]
    {
        crate::demo::settings_open()
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}
