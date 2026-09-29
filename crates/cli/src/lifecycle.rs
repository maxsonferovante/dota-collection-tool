//! Server lifecycle: foreground serve, detached spawn, stop and status.
//!
//! The same `serve_forever` path backs both modes: foreground blocks on it
//! until Ctrl-C, detached spawns this binary into it and records the PID.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::cli::UpArgs;
use crate::config;
use crate::paths::{self, Paths};

/// Health snapshot for `status`.
#[derive(Debug)]
pub struct StatusReport {
    pub running: bool,
    pub pid: Option<u32>,
    pub health: Option<String>,
}

fn read_pid(path: &Path) -> Result<Option<u32>> {
    match std::fs::read_to_string(path) {
        Ok(raw) => Ok(raw.trim().parse::<u32>().ok()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).context("cannot read pid file"),
    }
}

fn write_pid(path: &Path, pid: u32) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).context("cannot create state directory")?;
        }
    }
    std::fs::write(path, pid.to_string()).context("cannot write pid file")?;
    Ok(())
}

/// True when the OS still lists the process.
pub fn pid_alive(pid: u32) -> bool {
    let Ok(sys_pid) = pid.to_string().parse::<Pid>() else {
        return false;
    };
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
    system.process(sys_pid).is_some()
}

/// Ask the OS to terminate the process.
pub fn kill_pid(pid: u32) -> bool {
    let Ok(sys_pid) = pid.to_string().parse::<Pid>() else {
        return false;
    };
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
    system
        .process(sys_pid)
        .map(|process| process.kill())
        .unwrap_or(false)
}

/// Minimal blocking-free health probe: raw GET, body after the blank line.
pub async fn fetch_health(port: u16) -> Result<String> {
    let mut stream = tokio::time::timeout(
        Duration::from_secs(2),
        TcpStream::connect(("127.0.0.1", port)),
    )
    .await
    .context("health probe timed out")?
    .context("cannot connect for health probe")?;
    stream
        .write_all(b"GET /health HTTP/1.0\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await
        .context("cannot send health probe")?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .await
        .context("cannot read health probe")?;
    let text = String::from_utf8_lossy(&raw);
    let (_, body) = text.split_once("\r\n\r\n").context("bad health response")?;
    if !text.starts_with("HTTP/1.0 200") && !text.starts_with("HTTP/1.1 200") {
        bail!("unhealthy: {body}");
    }
    Ok(body.trim().to_owned())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

/// Open the store, replay overflow, then serve until Ctrl-C.
async fn serve_forever(port: u16, paths: &Paths, token: String) -> Result<()> {
    let store = dct_store::Store::open(&paths.db)
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let replayed = dct_store::replay(&store, &paths.overflow)
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    tracing::info!(
        inserted = replayed.inserted,
        skipped = replayed.skipped,
        "overflow replayed"
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!(%addr, "serving");
    dct_net::run(
        addr,
        store,
        dct_net::IngestConfig {
            token,
            overflow_path: paths.overflow.clone(),
            queue_capacity: dct_net::DEFAULT_QUEUE_CAPACITY,
        },
        shutdown_signal(),
    )
    .await
    .map_err(|err| anyhow::anyhow!("{err}"))?;
    Ok(())
}

/// Run `up`: foreground blocks, detached spawns this binary and records it.
pub async fn up(
    args: &UpArgs,
    exe: &Path,
    port: u16,
    db_override: Option<PathBuf>,
) -> Result<Option<u32>> {
    let paths = paths::resolve(db_override)?;
    let token = config::ensure_token(&paths.config).await?;
    if !args.detach {
        serve_forever(port, &paths, token).await?;
        return Ok(None);
    }
    if let Some(pid) = read_pid(&paths.pid)? {
        if pid_alive(pid) {
            bail!("server already running as pid {pid}");
        }
    }
    let log = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.log)
        .await
        .with_context(|| format!("cannot open {}", paths.log.display()))?
        .into_std()
        .await;
    let db_arg = paths.db.to_string_lossy().into_owned();
    let child = std::process::Command::new(exe)
        .arg("--port")
        .arg(port.to_string())
        .arg("--db")
        .arg(db_arg)
        .arg("up")
        .stdin(Stdio::null())
        .stdout(log.try_clone().context("cannot tee server log")?)
        .stderr(log)
        .spawn()
        .context("cannot spawn detached server")?;
    let pid = child.id();
    write_pid(&paths.pid, pid)?;
    if wait_healthy(port).await.is_err() {
        kill_pid(pid);
        std::fs::remove_file(&paths.pid).ok();
        bail!("detached server pid {pid} never became healthy");
    }
    println!("server detached as pid {pid}");
    Ok(Some(pid))
}

/// Poll the health endpoint until it answers or attempts run out.
async fn wait_healthy(port: u16) -> Result<()> {
    for _ in 0..50 {
        if fetch_health(port).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    bail!("health endpoint did not answer")
}

/// Run `down`: stop the detached server and clear the PID file.
pub async fn down(db_override: Option<PathBuf>) -> Result<bool> {
    let paths = paths::resolve(db_override)?;
    let pid = match read_pid(&paths.pid)? {
        Some(pid) => pid,
        None => {
            if paths.pid.is_file() {
                std::fs::remove_file(&paths.pid).ok();
            }
            println!("server is not running");
            return Ok(false);
        }
    };
    if !pid_alive(pid) {
        std::fs::remove_file(&paths.pid).ok();
        println!("server is not running (stale pid {pid} cleared)");
        return Ok(false);
    }
    if !kill_pid(pid) {
        bail!("cannot stop pid {pid}");
    }
    for _ in 0..30 {
        if !pid_alive(pid) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    std::fs::remove_file(&paths.pid).ok();
    println!("server pid {pid} stopped");
    Ok(true)
}

/// Run `status`: PID liveness plus the health endpoint.
pub async fn status(db_override: Option<PathBuf>, port: u16) -> Result<StatusReport> {
    let paths = paths::resolve(db_override)?;
    let pid = read_pid(&paths.pid)?;
    let alive = pid.is_some_and(pid_alive);
    let health = fetch_health(port).await.ok();
    let running = alive || health.is_some();
    let report = StatusReport {
        running,
        pid: pid.filter(|_| alive),
        health,
    };
    if report.running {
        println!("server is running");
    } else {
        println!("server is stopped");
    }
    Ok(report)
}
