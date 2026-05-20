use std::process::ExitCode;
use std::sync::Arc;

use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::RwLock;
use tracing::{info, warn};

use sni::{ITEM_PATH, Item, MENU_PATH, SharedState, StatusNotifierWatcherProxy};
use state::TrayState;

mod config;
mod ipc;
mod menu;
mod notify;
mod sni;
mod state;
mod watch;

const PROG: &str = "podctl-tray";

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    install_default_sigpipe();
    init_tracing();

    match run().await {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{PROG}: {e}");
            ExitCode::from(70)
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let cfg = config::load();
    info!(
        left_click = cfg.left_click.as_str(),
        low_battery = cfg.low_battery_threshold,
        notify = cfg.notify_threshold,
        "tray config loaded"
    );
    let state: SharedState = Arc::new(RwLock::new(TrayState::new(cfg.low_battery_threshold)));
    let quit = Arc::new(tokio::sync::Notify::new());

    let pid = std::process::id();
    let well_known = format!("org.kde.StatusNotifierItem-{pid}-1");

    let conn = zbus::connection::Builder::session()
        .map_err(|e| anyhow::anyhow!("session bus: {e}"))?
        .name(well_known.as_str())
        .map_err(|e| anyhow::anyhow!("request bus name: {e}"))?
        .serve_at(ITEM_PATH, Item::new(Arc::clone(&state), cfg.left_click))
        .map_err(|e| anyhow::anyhow!("export {ITEM_PATH}: {e}"))?
        .serve_at(
            MENU_PATH,
            menu::Menu::new(Arc::clone(&state), Arc::clone(&quit)),
        )
        .map_err(|e| anyhow::anyhow!("export {MENU_PATH}: {e}"))?
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("connect to session bus: {e}"))?;

    let watcher = StatusNotifierWatcherProxy::new(&conn).await.map_err(|e| {
        anyhow::anyhow!(
            "no StatusNotifierWatcher on the bus: {e} \
             (GNOME without an extension does not provide one — see 'podctl tray status')"
        )
    })?;
    watcher
        .register_status_notifier_item(&well_known)
        .await
        .map_err(|e| anyhow::anyhow!("register with watcher: {e}"))?;

    info!(name = %well_known, "podctl-tray registered");

    let notifier = match notify::Notifier::new(&conn, cfg.notify_threshold).await {
        Ok(n) => Some(Arc::new(n)),
        Err(e) => {
            warn!(error = %e, "no org.freedesktop.Notifications on the bus, low-battery alerts disabled");
            None
        }
    };

    watch::spawn(Arc::clone(&state), conn.clone(), notifier);

    wait_for_shutdown(&quit).await;
    info!("shutting down");
    Ok(())
}

async fn wait_for_shutdown(quit: &tokio::sync::Notify) {
    let mut term = match signal(SignalKind::terminate()) {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "cannot install SIGTERM handler, falling back to SIGINT only");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = quit.notified() => {}
            }
            return;
        }
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
        _ = quit.notified() => {}
    }
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();
}

fn install_default_sigpipe() {
    unsafe {
        let _ = libc_signal(SIGPIPE, SIG_DFL);
    }
}

const SIGPIPE: i32 = 13;
const SIG_DFL: usize = 0;
unsafe extern "C" {
    #[link_name = "signal"]
    fn libc_signal(sig: i32, handler: usize) -> usize;
}
