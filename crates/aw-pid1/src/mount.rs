//! Early mounts for PID 1

use anyhow::{Context, Result};
use log::info;
use nix::mount::{mount, MsFlags};
use std::path::Path;

pub fn early_mounts() -> Result<()> {
    info!("performing early mounts");

    mount_if_needed("/proc", "proc", "proc")?;
    mount_if_needed("/sys", "sysfs", "sysfs")?;
    mount_if_needed("/dev", "devtmpfs", "devtmpfs")?;
    mount_if_needed("/run", "tmpfs", "tmpfs")?;

    // Create essential dirs
    for dir in &["/run/azurewrath", "/run/systemd", "/run/dbus", "/var/log", "/var/lib/systemd"] {
        let _ = std::fs::create_dir_all(dir);
    }

    Ok(())
}

fn mount_if_needed(target: &str, fstype: &str, source: &str) -> Result<()> {
    let path = Path::new(target);
    if path.exists() && is_mounted(path) {
        info!("{} already mounted", target);
        return Ok(());
    }
    info!("mounting {} as {}", target, fstype);
    mount(
        Some(source),
        target,
        Some(fstype),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
        None::<&str>,
    )
    .with_context(|| format!("mount {}", target))?;
    Ok(())
}

fn is_mounted(path: &Path) -> bool {
    // Simple check: try to stat and see if it's a mountpoint via /proc/mounts
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        let target = path.to_string_lossy();
        mounts.lines().any(|line| line.contains(&*target))
    } else {
        false
    }
}
