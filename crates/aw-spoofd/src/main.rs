//! aw-spoofd - systemd and logind spoof daemon
//!
//! Implements enough of org.freedesktop.systemd1 and org.freedesktop.login1
//! so that GNOME 49+, Plasma Login Manager, and DrKonqi believe systemd is present.
//! Non-blocking where possible. Talks to real dbus-daemon.

use anyhow::{Context, Result};
use log::{error, info, warn};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use zbus::{connection, interface, Connection, Result as ZResult};
use zvariant::{OwnedValue, Value};

mod coredump;
mod logind;
mod logind_ext;
mod notify;
mod systemd;
mod systemd_ext;
mod systemd_more;
mod userdb;
mod dbus_helpers;

use coredump::CoredumpManager;
use logind::LogindManager;
use notify::NotifyListener;
use systemd::SystemdManager;
use userdb::UserDb;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("aw-spoofd starting");

    // Create runtime directories
    for dir in &[
        "/run/systemd",
        "/run/systemd/system",
        "/run/systemd/seats",
        "/run/systemd/sessions",
        "/run/systemd/users",
        "/var/lib/systemd/coredump",
    ] {
        let _ = std::fs::create_dir_all(dir);
    }

    // Start notify socket listener
    let notify = NotifyListener::start("/run/systemd/notify").await?;

    // Connect to system bus (real dbus-daemon)
    let conn = connection::Builder::system()?
        .name("org.freedesktop.systemd1")?
        .serve_at("/org/freedesktop/systemd1", SystemdManager::new())?
        .serve_at("/org/freedesktop/login1", LogindManager::new())?
        .build()
        .await
        .context("failed to connect to system bus")?;

    info!("registered on system bus as systemd1 and login1");

    // Userdb fallback
    let _userdb = UserDb::new();

    // Coredump manager
    let _coredump = CoredumpManager::new("/var/lib/systemd/coredump");

    // Keep running
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        // Periodic maintenance
    }
}
