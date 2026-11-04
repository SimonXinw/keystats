#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod board;
mod board_test;
#[cfg(debug_assertions)]
mod demo;
mod export;
mod hook;
mod instance;
mod keys;
mod launch;
mod live;
#[cfg(test)]
mod live_test;
mod pace;
mod palette;
mod process;
mod stats;
mod stats_test;
mod store;
mod theme;
mod ui_dialog;
mod ui_header;
mod ui_heat;
mod ui_grid;
mod ui_kit;
mod ui_list;
mod ui_mouse;
mod ui_paint;
mod ui_palette;
mod ui_table;

fn main() -> eframe::Result<()> {
    if !instance::primary() {
        return Ok(());
    }
    launch::run()
}
