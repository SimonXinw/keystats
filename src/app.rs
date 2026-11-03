use crate::export;
use crate::hook::Hooks;
use crate::stats::Stats;
use crate::store;
use crate::theme;
use crate::ui_header;
use crate::ui_table::{self, ReportView};
use chrono::Local;
use std::sync::Arc;
use std::time::Duration;

pub struct KeyStatsApp {
    hooks: Option<Hooks>,
    stats: Arc<Stats>,
    keeper: store::Keeper,
    filter: String,
    view: ReportView,
    confirm_reset: bool,
    notice: String,
    error: Option<String>,
}

impl KeyStatsApp {
    fn new() -> Self {
        let stats = Stats::new();
        let (keeper, restored) = store::Keeper::open(&stats);
        let (hooks, error) = match Hooks::start(Arc::clone(&stats)) {
            Ok(hooks) => (Some(hooks), None),
            Err(err) => (None, Some(err)),
        };
        #[cfg(debug_assertions)]
        crate::demo::seed(&stats);
        #[cfg(debug_assertions)]
        let view = crate::demo::view().unwrap_or(ReportView::Detail);
        #[cfg(not(debug_assertions))]
        let view = ReportView::Detail;
        #[cfg(debug_assertions)]
        let confirm_reset = crate::demo::reset_open();
        #[cfg(not(debug_assertions))]
        let confirm_reset = false;
        Self {
            hooks,
            stats,
            keeper,
            filter: String::new(),
            view,
            confirm_reset,
            notice: restored,
            error,
        }
    }

    fn apply(&mut self, action: ui_header::HeaderAction, rows: &[crate::stats::Row]) {
        if action.start {
            if let Some(hooks) = &self.hooks {
                hooks.clear_keyboard_state();
            }
            self.stats.start();
            self.notice.clear();
        }
        if action.pause {
            self.stats.pause();
            self.keeper.flush(&self.stats);
        }
        if action.reset {
            self.confirm_reset = true;
        }
        if action.export {
            self.export_csv(rows);
        }
    }

    fn export_csv(&mut self, rows: &[crate::stats::Row]) {
        let name = format!("keystats-{}.csv", Local::now().format("%Y%m%d-%H%M%S"));
        let Some(path) = rfd::FileDialog::new()
            .set_title("导出表格")
            .set_file_name(name)
            .add_filter("表格", &["csv"])
            .save_file()
        else {
            return;
        };
        self.notice = match export::write_csv(&path, rows) {
            Ok(()) => format!("已导出 {}", path.display()),
            Err(err) => format!("导出失败：{err}"),
        };
    }
}

impl eframe::App for KeyStatsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(300));
        #[cfg(debug_assertions)]
        crate::demo::capture(ctx);
        self.keeper.tick(&self.stats);
        let mut count_self = self.stats.count_self();
        let snap = self.stats.snapshot();
        let ready = self.hooks.is_some();
        let mut action = ui_header::HeaderAction {
            start: false,
            pause: false,
            reset: false,
            export: false,
        };
        let frame = egui::Frame::NONE
            .fill(theme::BG)
            .inner_margin(egui::Margin::symmetric(24, 20));
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            action = ui_header::show(ui, &snap, ready);
            ui.add_space(theme::GAP + 4.0);
            let status = match (&self.error, self.notice.is_empty()) {
                (Some(error), _) => Some(ui_table::Status {
                    text: error,
                    color: theme::DANGER,
                }),
                (None, false) => Some(ui_table::Status {
                    text: &self.notice,
                    color: if self.notice.starts_with("导出失败") {
                        theme::DANGER
                    } else {
                        theme::ACCENT
                    },
                }),
                (None, true) => None,
            };
            ui_table::show(
                ui,
                &mut self.filter,
                &mut self.view,
                &mut count_self,
                &snap,
                status,
            );
        });
        if count_self != self.stats.count_self() {
            self.stats.set_count_self(count_self);
            self.keeper.flush(&self.stats);
        }
        self.apply(action, &snap.rows);
        self.reset_dialog(ctx);
    }
}

impl KeyStatsApp {
    fn reset_dialog(&mut self, ctx: &egui::Context) {
        if !self.confirm_reset {
            return;
        }
        match crate::ui_dialog::confirm_reset(ctx) {
            crate::ui_dialog::ResetChoice::Stay => {}
            crate::ui_dialog::ResetChoice::Cancel => self.confirm_reset = false,
            crate::ui_dialog::ResetChoice::Confirm => {
                self.stats.reset();
                if let Some(hooks) = &self.hooks {
                    hooks.clear_keyboard_state();
                }
                self.keeper.flush(&self.stats);
                self.confirm_reset = false;
                self.notice.clear();
            }
        }
    }
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(crate::instance::TITLE)
            .with_icon(app_icon())
            .with_inner_size([1500.0, 880.0])
            .with_min_inner_size([1280.0, 680.0]),
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        crate::instance::TITLE,
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(KeyStatsApp::new()))
        }),
    )
}

impl Drop for KeyStatsApp {
    fn drop(&mut self) {
        self.hooks.take();
        self.keeper.flush(&self.stats);
    }
}

fn app_icon() -> egui::IconData {
    egui::IconData {
        rgba: include_bytes!("../assets/icon_64.rgba").to_vec(),
        width: 64,
        height: 64,
    }
}
