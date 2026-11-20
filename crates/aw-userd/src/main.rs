//! aw-userd - user session manager
//! Like systemd --user. Manages user services and XDG autostart.

use anyhow::{Context, Result};
use log::{info, warn};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use xdg::BaseDirectories;

mod services;
mod unit;

use services::UserServiceManager;

fn main() -> Result<()> {
    env_logger::init();
    info!("aw-userd starting for user");

    let uid = nix::unistd::getuid().as_raw();
    let runtime_dir = format!("/run/user/{}", uid);
    std::fs::create_dir_all(&runtime_dir).ok();
    env::set_var("XDG_RUNTIME_DIR", &runtime_dir);

    // Setup session bus address if not set
    if env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
        let addr = format!("unix:path={}/bus", runtime_dir);
        env::set_var("DBUS_SESSION_BUS_ADDRESS", &addr);
        // Start a private dbus if needed
        let _ = Command::new("dbus-daemon")
            .args(["--session", "--address", &addr, "--nofork"])
            .spawn();
    }

    let mut mgr = UserServiceManager::new()?;
    mgr.load_autostart()?;
    mgr.run_event_loop()?;

    Ok(())
}
