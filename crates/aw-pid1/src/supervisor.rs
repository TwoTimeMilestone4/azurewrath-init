//! Extended process control and supervision helpers

use super::ChildState;
use anyhow::Result;
use log::{info, warn};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::collections::HashMap;

/// Supervisor state for tracking long-running children
pub struct Supervisor {
    pub children: HashMap<Pid, ChildState>,
    pub restart_counts: HashMap<String, u32>,
    pub max_restarts: u32,
}

impl Supervisor {
    pub fn new() -> Self {
        Supervisor {
            children: HashMap::new(),
            restart_counts: HashMap::new(),
            max_restarts: 10,
        }
    }

    pub fn register(&mut self, pid: Pid, state: ChildState) {
        self.children.insert(pid, state);
    }

    pub fn on_exit(&mut self, pid: Pid) -> Option<ChildState> {
        self.children.remove(&pid)
    }

    pub fn should_restart(&mut self, id: &str) -> bool {
        let count = self.restart_counts.entry(id.to_string()).or_insert(0);
        *count += 1;
        if *count > self.max_restarts {
            warn!("max restarts exceeded for {}", id);
            false
        } else {
            true
        }
    }

    pub fn kill_all(&self, sig: Signal) {
        for pid in self.children.keys() {
            let _ = kill(*pid, sig);
        }
    }

    pub fn active_count(&self) -> usize {
        self.children.len()
    }
}

/// Send signal to a process group
pub fn signal_group(pgid: Pid, sig: Signal) -> Result<()> {
    kill(pgid, sig)?;
    Ok(())
}

/// Check if a pid is still alive
pub fn is_alive(pid: Pid) -> bool {
    kill(pid, None).is_ok()
}
