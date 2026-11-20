//! Process management for PID 1
//! Async spawn, no blocking wait.

use anyhow::{Context, Result};
use log::info;
use nix::unistd::{fork, ForkResult, Pid};
use std::process::Command;

use crate::inittab::InitEntry;

#[derive(Debug, Clone)]
pub struct ChildState {
    pub id: String,
    pub entry: InitEntry,
    pub respawn: bool,
}

pub fn spawn_async(cmdline: &str, track_respawn: bool) -> Result<Pid> {
    info!("spawning (async): {}", cmdline);

    // Parse simple command line
    let parts: Vec<&str> = cmdline.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("empty command"));
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            info!("forked child {} for {}", child, cmdline);
            Ok(child)
        }
        Ok(ForkResult::Child) => {
            // Child: exec
            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }
            // Redirect stdio to /dev/console or null
            let _ = cmd
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());

            let err = cmd.exec();
            // If exec fails
            eprintln!("exec failed: {}", err);
            std::process::exit(127);
        }
        Err(e) => Err(e).context("fork"),
    }
}

pub fn spawn_and_track(entry: &InitEntry) -> Result<(Pid, ChildState)> {
    let pid = spawn_async(&entry.process, true)?;
    let state = ChildState {
        id: entry.id.clone(),
        entry: entry.clone(),
        respawn: true,
    };
    Ok((pid, state))
}
