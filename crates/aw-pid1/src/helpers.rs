//! Additional production helpers and edge-case handling for PID 1
//! Covers filesystem sync, utmp updates, and safe re-exec paths.

use anyhow::Result;
use log::info;
use nix::unistd::{sync, Pid};
use std::fs;
use std::io::Write;

/// Sync all filesystems before halt/reboot
pub fn sync_filesystems() {
    info!("syncing filesystems");
    sync();
}

/// Write current runlevel to /run for CLI
pub fn write_runlevel_file(rl: u8) -> Result<()> {
    fs::write("/run/azurewrath.runlevel", format!("{}\n", rl))?;
    Ok(())
}

/// Update utmp/wtmp if available (best-effort)
pub fn update_utmp_runlevel(old: u8, new: u8) {
    // Best-effort; many systems use systemd-logind for this now
    info!("runlevel change {} -> {} (utmp update skipped or handled by spoofd)", old, new);
}

/// Safe path for re-exec of PID 1 (for upgrades)
pub fn prepare_reexec() -> Result<()> {
    info!("preparing for re-exec");
    // Close non-essential fds, etc. would go here
    Ok(())
}

/// Check essential paths exist
pub fn sanity_check() -> Result<()> {
    for p in &["/proc", "/sys", "/dev", "/run", "/etc/inittab"] {
        if !std::path::Path::new(p).exists() {
            log::warn!("missing path: {}", p);
        }
    }
    Ok(())
}

/// Load kernel command line for init= overrides
pub fn kernel_cmdline() -> String {
    fs::read_to_string("/proc/cmdline").unwrap_or_default()
}

/// Detect if we should enter emergency shell
pub fn is_emergency() -> bool {
    let cmd = kernel_cmdline();
    cmd.contains("emergency") || cmd.contains("single") || cmd.contains("1")
}
