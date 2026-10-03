//! Graphical manager for the local game-state collector.
//!
//! Double-click entry point for non-terminal users: collector status with
//! start/stop, Dota folder picker plus one-click install, and JSON export
//! with an output-folder picker. All work reuses the `dct-cli` library, so
//! the window and the CLI operate on the same state directory.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("dct-gui-worker")
            .build()
            .expect("cannot start background runtime"),
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([580.0, 720.0])
            .with_min_inner_size([480.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Dota Collection Tool",
        options,
        Box::new(move |ctx| Ok(Box::new(app::App::new(ctx, runtime)))),
    )
}
