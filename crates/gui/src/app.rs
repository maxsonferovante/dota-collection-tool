//! Window state: three panels (service, install, export) over background jobs.
//!
//! Long work never blocks the UI thread: each action spawns onto the shared
//! Tokio runtime and reports back through an `mpsc` channel polled in
//! `update`. The collector itself runs in-process (same state directory the
//! CLI uses), so the window and `dct` stay interoperable.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use dct_cli::cli::{ExportArgs, InstallArgs};
use dct_cli::commands::{export, install};
use dct_cli::{config, lifecycle, paths, steam};

const DEFAULT_PORT: &str = "53000";
const STATUS_POLL_INTERVAL: Duration = Duration::from_secs(3);
const STOP_GRACE_PERIOD: Duration = Duration::from_secs(6);

/// Point-in-time service state for the status panel.
struct StatusSnapshot {
    running: bool,
    pid: Option<u32>,
    health: Option<String>,
}

/// In-process collector handles: the stop signal plus an abort fallback.
struct RunningServer {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    abort: tokio::task::AbortHandle,
}

/// One-line feedback under a panel: green on success, red on failure.
#[derive(Default)]
struct Message {
    text: String,
    ok: bool,
}

impl Message {
    fn set(&mut self, text: String, ok: bool) {
        self.text = text;
        self.ok = ok;
    }

    fn show(&self, ui: &mut egui::Ui) {
        if self.text.is_empty() {
            return;
        }
        let color = if self.ok {
            egui::Color32::DARK_GREEN
        } else {
            egui::Color32::DARK_RED
        };
        ui.colored_label(color, &self.text);
    }
}

/// Parse the port field; spaces in paths are fine, ports must be numbers.
fn parse_port(text: &str) -> Result<u16, String> {
    text.trim()
        .parse::<u16>()
        .map_err(|_| format!("port must be a number 1-65535, got {text:?}"))
}

/// Non-empty CLI filter: blank means "no filter".
fn opt(text: &str) -> Option<String> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Serve until the stop signal fires; returns a human-readable summary.
async fn serve_forever(
    port: u16,
    stop: tokio::sync::oneshot::Receiver<()>,
) -> Result<String, String> {
    if lifecycle::fetch_health(port).await.is_ok() {
        return Err(format!(
            "something already answers on port {port} (another collector running?)"
        ));
    }
    let resolved =
        paths::resolve(None).map_err(|err| format!("cannot resolve state dir: {err:#}"))?;
    let token = config::ensure_token(&resolved.config)
        .await
        .map_err(|err| format!("cannot load token: {err:#}"))?;
    let store = dct_store::Store::open(&resolved.db)
        .await
        .map_err(|err| format!("cannot open database: {err}"))?;
    let replayed = dct_store::replay(&store, &resolved.overflow)
        .await
        .map_err(|err| format!("cannot replay overflow: {err}"))?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    dct_net::run(
        addr,
        store,
        dct_net::IngestConfig {
            token,
            overflow_path: resolved.overflow.clone(),
            queue_capacity: dct_net::DEFAULT_QUEUE_CAPACITY,
        },
        async move {
            let _ = stop.await;
        },
    )
    .await
    .map_err(|err| format!("collector failed: {err}"))?;
    Ok(format!(
        "Collector stopped (overflow replayed {} inserted, {} skipped).",
        replayed.inserted, replayed.skipped
    ))
}

/// Default export directory: `exports` next to the database.
fn default_export_dir() -> String {
    paths::resolve(None)
        .map(|resolved| {
            resolved
                .db
                .parent()
                .map(|dir| dir.join("exports"))
                .unwrap_or_else(|| PathBuf::from("exports"))
                .display()
                .to_string()
        })
        .unwrap_or_else(|_| String::from("exports"))
}

/// Reveal a folder in the platform file manager.
fn open_folder(path: &str) {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}

pub struct App {
    runtime: Arc<tokio::runtime::Runtime>,
    port_text: String,
    dota_dir: String,
    config_name: String,
    overwrite: bool,
    export_dir: String,
    export_match: String,
    export_kind: String,
    auto_refresh: bool,
    running: bool,
    pid: Option<u32>,
    health: Option<String>,
    last_status: Option<Instant>,
    stopping_since: Option<Instant>,
    server: Option<RunningServer>,
    server_msg: Message,
    install_msg: Message,
    export_msg: Message,
    db_location: String,
    status_rx: Option<mpsc::Receiver<Result<StatusSnapshot, String>>>,
    install_rx: Option<mpsc::Receiver<Result<String, String>>>,
    export_rx: Option<mpsc::Receiver<Result<String, String>>>,
    server_rx: Option<mpsc::Receiver<Result<String, String>>>,
}

impl App {
    pub fn new(ctx: &eframe::CreationContext<'_>, runtime: Arc<tokio::runtime::Runtime>) -> Self {
        let dota_dir = steam::find_dota_root()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let db_location = paths::resolve(None)
            .map(|resolved| resolved.db.display().to_string())
            .unwrap_or_else(|_| String::from("unknown"));
        let mut app = Self {
            runtime,
            port_text: DEFAULT_PORT.to_owned(),
            dota_dir,
            config_name: String::from("dct"),
            overwrite: true,
            export_dir: default_export_dir(),
            export_match: String::new(),
            export_kind: String::new(),
            auto_refresh: true,
            running: false,
            pid: None,
            health: None,
            last_status: None,
            stopping_since: None,
            server: None,
            server_msg: Message::default(),
            install_msg: Message::default(),
            export_msg: Message::default(),
            db_location,
            status_rx: None,
            install_rx: None,
            export_rx: None,
            server_rx: None,
        };
        app.refresh_status();
        ctx.egui_ctx.request_repaint();
        app
    }

    fn status_busy(&self) -> bool {
        self.status_rx.is_some()
    }

    fn install_busy(&self) -> bool {
        self.install_rx.is_some()
    }

    fn export_busy(&self) -> bool {
        self.export_rx.is_some()
    }

    fn server_busy(&self) -> bool {
        self.server_rx.is_some()
    }

    /// Drain finished background jobs into panel state.
    fn poll_jobs(&mut self) {
        if let Some(rx) = &self.status_rx {
            if let Ok(outcome) = rx.try_recv() {
                self.status_rx = None;
                match outcome {
                    Ok(snapshot) => {
                        self.running = snapshot.running;
                        self.pid = snapshot.pid;
                        self.health = snapshot.health;
                    }
                    Err(err) => {
                        self.running = false;
                        self.pid = None;
                        self.health = Some(err);
                    }
                }
            }
        }
        if let Some(rx) = &self.install_rx {
            if let Ok(outcome) = rx.try_recv() {
                self.install_rx = None;
                match outcome {
                    Ok(summary) => self.install_msg.set(summary, true),
                    Err(err) => self.install_msg.set(err, false),
                }
            }
        }
        if let Some(rx) = &self.export_rx {
            if let Ok(outcome) = rx.try_recv() {
                self.export_rx = None;
                match outcome {
                    Ok(summary) => self.export_msg.set(summary, true),
                    Err(err) => self.export_msg.set(err, false),
                }
            }
        }
        if let Some(rx) = &self.server_rx {
            if let Ok(outcome) = rx.try_recv() {
                self.server_rx = None;
                let stopping = self.stopping_since.take().is_some();
                self.server = None;
                match outcome {
                    Ok(summary) => self.server_msg.set(summary, true),
                    Err(err) => {
                        if stopping {
                            self.server_msg
                                .set(String::from("Collector stopped."), true);
                        } else {
                            self.server_msg.set(err, false);
                        }
                    }
                }
                self.refresh_status();
            }
        }
    }

    /// Ask the runtime for a fresh status snapshot (no-op while one is due).
    fn refresh_status(&mut self) {
        if self.status_rx.is_some() {
            return;
        }
        let port = match parse_port(&self.port_text) {
            Ok(port) => port,
            Err(err) => {
                self.running = false;
                self.health = Some(err);
                return;
            }
        };
        let (tx, rx) = mpsc::channel();
        self.status_rx = Some(rx);
        self.last_status = Some(Instant::now());
        self.runtime.spawn(async move {
            let outcome = match lifecycle::status(None, port).await {
                Ok(report) => Ok(StatusSnapshot {
                    running: report.running,
                    pid: report.pid,
                    health: report.health,
                }),
                Err(err) => Err(format!("status check failed: {err:#}")),
            };
            let _ = tx.send(outcome);
        });
    }

    /// Start the in-process collector.
    fn start_server(&mut self) {
        if self.server.is_some() || self.server_busy() {
            return;
        }
        let port = match parse_port(&self.port_text) {
            Ok(port) => port,
            Err(err) => {
                self.server_msg.set(err, false);
                return;
            }
        };
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        let (tx, rx) = mpsc::channel();
        self.server_rx = Some(rx);
        let join = self
            .runtime
            .spawn(async move { serve_forever(port, stop_rx).await });
        let abort = join.abort_handle();
        self.runtime.spawn(async move {
            let outcome = match join.await {
                Ok(inner) => inner,
                Err(err) => Err(format!("collector task failed: {err}")),
            };
            let _ = tx.send(outcome);
        });
        self.server = Some(RunningServer {
            stop: Some(stop_tx),
            abort,
        });
        self.server_msg
            .set(format!("Starting collector on port {port}..."), true);
    }

    /// Stop the in-process collector, or a detached CLI server if none.
    fn stop_server(&mut self) {
        if let Some(server) = self.server.as_mut() {
            if let Some(stop) = server.stop.take() {
                let _ = stop.send(());
            }
            self.stopping_since = Some(Instant::now());
            self.server_msg
                .set(String::from("Stopping collector..."), true);
            return;
        }
        if self.server_busy() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.server_rx = Some(rx);
        self.server_msg
            .set(String::from("Stopping detached server..."), true);
        self.runtime.spawn(async move {
            let outcome = match lifecycle::down(None).await {
                Ok(true) => Ok(String::from("Detached server stopped.")),
                Ok(false) => Ok(String::from("Server is not running.")),
                Err(err) => Err(format!("cannot stop server: {err:#}")),
            };
            let _ = tx.send(outcome);
        });
    }

    /// Abort a stuck collector task.
    fn force_stop(&mut self) {
        if let Some(server) = &self.server {
            server.abort.abort();
        }
        self.stopping_since = Some(Instant::now());
        self.server_msg
            .set(String::from("Forcing collector shutdown..."), true);
    }

    /// Write the game config into the chosen Dota folder.
    fn install_config(&mut self) {
        if self.install_busy() {
            return;
        }
        let dir = self.dota_dir.trim().to_owned();
        if dir.is_empty() {
            self.install_msg.set(
                String::from("Pick the Dota install folder first (Browse or Auto-detect)."),
                false,
            );
            return;
        }
        let path = PathBuf::from(&dir);
        if !path.is_dir() {
            self.install_msg
                .set(format!("Folder does not exist: {}", path.display()), false);
            return;
        }
        if !path.join("game").join("dota").is_dir() {
            self.install_msg.set(
                String::from(
                    "That folder does not look like a Dota install (missing game/dota). \
                     Pick the \"dota 2 beta\" folder.",
                ),
                false,
            );
            return;
        }
        let port = match parse_port(&self.port_text) {
            Ok(port) => port,
            Err(err) => {
                self.install_msg.set(err, false);
                return;
            }
        };
        let args = InstallArgs {
            name: self.config_name.trim().to_owned(),
            dota_dir: Some(path),
            token: None,
            force: self.overwrite,
        };
        let (tx, rx) = mpsc::channel();
        self.install_rx = Some(rx);
        self.install_msg
            .set(String::from("Writing game config..."), true);
        self.runtime.spawn(async move {
            let outcome = match install::run(&args, port, None).await {
                Ok(written) => Ok(format!("Game config written to {}", written.display())),
                Err(err) => Err(format!("install failed: {err:#}")),
            };
            let _ = tx.send(outcome);
        });
    }

    /// Dump frames/happenings JSON into the chosen folder.
    fn run_export(&mut self) {
        if self.export_busy() {
            return;
        }
        let out = self.export_dir.trim();
        let args = ExportArgs {
            out: (!out.is_empty()).then(|| PathBuf::from(out)),
            match_id: opt(&self.export_match),
            kind: opt(&self.export_kind),
        };
        let (tx, rx) = mpsc::channel();
        self.export_rx = Some(rx);
        self.export_msg.set(String::from("Exporting..."), true);
        self.runtime.spawn(async move {
            let outcome = match export::run(&args, None).await {
                Ok(report) => Ok(format!(
                    "Exported {} frames and {} happenings to {}",
                    report.frames,
                    report.happenings,
                    report.dir.display()
                )),
                Err(err) => Err(format!("export failed: {err:#}")),
            };
            let _ = tx.send(outcome);
        });
    }

    fn status_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("1 · Collector service");
        ui.horizontal(|ui| {
            let (dot, label) = if self.server.is_some() || self.running {
                (egui::Color32::DARK_GREEN, "Running")
            } else {
                (egui::Color32::DARK_RED, "Stopped")
            };
            ui.colored_label(dot, "●");
            ui.label(label);
            if let Some(pid) = self.pid {
                ui.label(format!("(pid {pid})"));
            }
            if self.status_busy() {
                ui.spinner();
            }
        });
        if let Some(health) = &self.health {
            let one_line: String = health
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(140)
                .collect();
            if !one_line.is_empty() {
                ui.label(format!("Health: {one_line}"));
            }
        }
        ui.horizontal(|ui| {
            ui.label("Port:");
            ui.add(
                egui::TextEdit::singleline(&mut self.port_text)
                    .desired_width(70.0)
                    .hint_text(DEFAULT_PORT),
            );
            let start = ui.button("Start");
            if start.clicked() {
                self.start_server();
            }
            let stop = ui.button("Stop");
            if stop.clicked() {
                self.stop_server();
            }
            if ui.button("Refresh").clicked() {
                self.refresh_status();
            }
        });
        if let Some(since) = self.stopping_since {
            if since.elapsed() > STOP_GRACE_PERIOD && ui.button("Force stop").clicked() {
                self.force_stop();
            }
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.auto_refresh, "Auto-refresh status");
        });
        self.server_msg.show(ui);
    }

    fn install_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("2 · Dota install folder");
        ui.label("Pick the \"dota 2 beta\" folder, then install the game config.");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.dota_dir)
                    .desired_width(320.0)
                    .hint_text("E:\\...\\dota 2 beta"),
            );
            if ui.button("Browse...").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Select the \"dota 2 beta\" folder")
                    .pick_folder()
                {
                    self.dota_dir = folder.display().to_string();
                }
            }
            if ui.button("Auto-detect").clicked() {
                match steam::find_dota_root() {
                    Some(root) => {
                        self.dota_dir = root.display().to_string();
                        self.install_msg
                            .set(format!("Detected {}", self.dota_dir), true);
                    }
                    None => self.install_msg.set(
                        String::from("Dota not found automatically; pick the folder manually."),
                        false,
                    ),
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Config name:");
            ui.add(egui::TextEdit::singleline(&mut self.config_name).desired_width(90.0));
            ui.checkbox(&mut self.overwrite, "Overwrite existing (backup as .bak)");
            let install = ui.button("Install game config");
            if install.clicked() {
                self.install_config();
            }
            if self.install_busy() {
                ui.spinner();
            }
        });
        ui.label("After installing: add -gamestateintegration to the Steam launch options and restart the game.");
        self.install_msg.show(ui);
    }

    fn export_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("3 · Export JSON");
        ui.label("Save match frames and happenings as JSON files.");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.export_dir)
                    .desired_width(320.0)
                    .hint_text("Output folder"),
            );
            if ui.button("Browse...").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Choose where to save the export")
                    .pick_folder()
                {
                    self.export_dir = folder.display().to_string();
                }
            }
            if ui.button("Default").clicked() {
                self.export_dir = default_export_dir();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Match:");
            ui.add(
                egui::TextEdit::singleline(&mut self.export_match)
                    .desired_width(110.0)
                    .hint_text("optional"),
            );
            ui.label("Kind:");
            ui.add(
                egui::TextEdit::singleline(&mut self.export_kind)
                    .desired_width(110.0)
                    .hint_text("e.g. kill"),
            );
            let run = ui.button("Export");
            if run.clicked() {
                self.run_export();
            }
            if self.export_busy() {
                ui.spinner();
            }
            if ui.button("Open folder").clicked() {
                let target = if self.export_dir.trim().is_empty() {
                    default_export_dir()
                } else {
                    self.export_dir.trim().to_owned()
                };
                open_folder(&target);
            }
        });
        self.export_msg.show(ui);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_jobs();
        if self.auto_refresh
            && !self.status_busy()
            && self
                .last_status
                .is_none_or(|at| at.elapsed() > STATUS_POLL_INTERVAL)
        {
            self.refresh_status();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Dota Collection Tool");
                ui.label("Local game-state collector — no command line needed.");
                ui.separator();
                self.status_panel(ui);
                ui.separator();
                self.install_panel(ui);
                ui.separator();
                self.export_panel(ui);
                ui.separator();
                ui.collapsing("Details", |ui| {
                    ui.label(format!("Database: {}", self.db_location));
                    ui.label(
                        "State (database, token, logs) lives next to this app, \
                         shared with the dct command line.",
                    );
                });
            });
        });
        if self.status_busy() || self.install_busy() || self.export_busy() || self.server_busy() {
            ctx.request_repaint_after(Duration::from_millis(150));
        } else if self.auto_refresh {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_parsing_accepts_valid_ports() {
        assert_eq!(parse_port("53000"), Ok(53_000));
        assert_eq!(parse_port(" 80 "), Ok(80));
    }

    #[test]
    fn port_parsing_rejects_garbage() {
        assert!(parse_port("").is_err());
        assert!(parse_port("[OPTIONS]").is_err());
        assert!(parse_port("99999").is_err());
    }

    #[test]
    fn blank_filters_become_none() {
        assert_eq!(opt("  "), None);
        assert_eq!(opt("kill"), Some(String::from("kill")));
    }
}
