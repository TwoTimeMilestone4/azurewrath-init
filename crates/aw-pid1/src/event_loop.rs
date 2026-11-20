//! Non-blocking event loop for PID 1
//! signalfd + epoll. Handle one event -> return immediately.

use anyhow::{Context, Result};
use log::{error, info, warn};
use nix::sys::epoll::{epoll_create1, epoll_ctl, epoll_wait, EpollEvent, EpollFlags, EpollOp};
use nix::sys::signalfd::{signalfd, SfdFlags};
use nix::sys::signal::{SigSet, Signal};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;

use crate::inittab::{Action, InitEntry, InitTab};
use crate::process::{self, ChildState};
use crate::runlevel::RunlevelManager;
use crate::signals::SignalHandler;

pub struct EventLoop {
    epoll_fd: RawFd,
    signal_fd: RawFd,
    children: HashMap<Pid, ChildState>,
    respawn_map: HashMap<String, InitEntry>,
}

impl EventLoop {
    pub fn new() -> Result<Self> {
        let epoll_fd = epoll_create1(nix::sys::epoll::EpollCreateFlags::EPOLL_CLOEXEC)
            .context("epoll_create1")?;

        let mut mask = SigSet::empty();
        mask.add(Signal::SIGCHLD);
        mask.add(Signal::SIGINT);
        mask.add(Signal::SIGTERM);
        mask.add(Signal::SIGHUP);
        mask.add(Signal::SIGUSR1);
        mask.add(Signal::SIGUSR2);
        mask.add(Signal::SIGPWR);
        mask.add(Signal::SIGWINCH);

        // Block signals so signalfd receives them
        nix::sys::signal::sigprocmask(nix::sys::signal::SigmaskHow::SIG_BLOCK, Some(&mask), None)
            .context("sigprocmask")?;

        let signal_fd = signalfd(-1, &mask, SfdFlags::SFD_CLOEXEC | SfdFlags::SFD_NONBLOCK)
            .context("signalfd")?;

        let mut event = EpollEvent::new(EpollFlags::EPOLLIN, signal_fd.as_raw_fd() as u64);
        epoll_ctl(
            epoll_fd,
            EpollOp::EpollCtlAdd,
            signal_fd.as_raw_fd(),
            Some(&mut event),
        )
        .context("epoll_ctl signalfd")?;

        Ok(EventLoop {
            epoll_fd,
            signal_fd: signal_fd.as_raw_fd(),
            children: HashMap::new(),
            respawn_map: HashMap::new(),
        })
    }

    pub fn run(
        &mut self,
        inittab: &mut InitTab,
        rl_mgr: &mut RunlevelManager,
        sig_handler: &mut SignalHandler,
    ) -> Result<()> {
        info!("entering non-blocking event loop");

        // Initial runlevel transition
        self.transition_runlevel(rl_mgr.current(), inittab, rl_mgr)?;

        let mut events = [EpollEvent::empty(); 16];

        loop {
            let n = epoll_wait(self.epoll_fd, &mut events, 1000) // 1s timeout for periodic checks
                .context("epoll_wait")?;

            for i in 0..n {
                let ev = events[i];
                if ev.data() == self.signal_fd as u64 {
                    self.handle_signals(inittab, rl_mgr, sig_handler)?;
                }
            }

            // Periodic: check control file for runlevel requests
            self.check_control_file(rl_mgr, inittab)?;

            // Respawn crashed processes if needed
            self.handle_respawns()?;
        }
    }

    fn handle_signals(
        &mut self,
        inittab: &mut InitTab,
        rl_mgr: &mut RunlevelManager,
        sig_handler: &mut SignalHandler,
    ) -> Result<()> {
        while let Some(sig) = sig_handler.read_signal(self.signal_fd)? {
            match sig {
                Signal::SIGCHLD => {
                    self.reap_children()?;
                }
                Signal::SIGHUP => {
                    info!("SIGHUP: reloading inittab");
                    inittab.reload(std::path::Path::new("/etc/inittab"))?;
                    self.respawn_map.clear();
                }
                Signal::SIGINT | Signal::SIGTERM => {
                    info!("shutdown signal received");
                    self.transition_runlevel(0, inittab, rl_mgr)?;
                    // After halt scripts, poweroff
                    self.do_halt()?;
                }
                Signal::SIGUSR1 => {
                    info!("SIGUSR1: switch to single user");
                    self.transition_runlevel(1, inittab, rl_mgr)?;
                }
                Signal::SIGUSR2 => {
                    info!("SIGUSR2: re-exec or special");
                }
                Signal::SIGPWR => {
                    info!("SIGPWR: power fail");
                    // Run powerfail entries
                    for e in inittab.entries_for_runlevel(rl_mgr.current()) {
                        if matches!(e.action, Action::PowerFail) {
                            let _ = process::spawn_async(&e.process, false);
                        }
                    }
                }
                Signal::SIGWINCH => {
                    // Keyboard signal, often used for runlevel change
                    info!("SIGWINCH");
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn reap_children(&mut self) -> Result<()> {
        loop {
            match nix::sys::wait::waitpid(None, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
                Ok(nix::sys::wait::WaitStatus::Exited(pid, status)) => {
                    info!("child {} exited with {}", pid, status);
                    if let Some(state) = self.children.remove(&pid) {
                        if state.respawn {
                            // Schedule respawn
                            info!("scheduling respawn for {}", state.id);
                            self.respawn_map.insert(state.id.clone(), state.entry);
                        }
                    }
                }
                Ok(nix::sys::wait::WaitStatus::Signaled(pid, sig, _)) => {
                    info!("child {} signaled {:?}", pid, sig);
                    if let Some(state) = self.children.remove(&pid) {
                        if state.respawn {
                            self.respawn_map.insert(state.id.clone(), state.entry);
                        }
                    }
                }
                Ok(nix::sys::wait::WaitStatus::StillAlive) | Ok(_) => break,
                Err(nix::errno::Errno::ECHILD) => break,
                Err(e) => {
                    warn!("waitpid error: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_respawns(&mut self) -> Result<()> {
        let to_respawn: Vec<_> = self.respawn_map.drain().collect();
        for (id, entry) in to_respawn {
            info!("respawning {}", id);
            if let Ok(pid) = process::spawn_async(&entry.process, true) {
                self.children.insert(
                    pid,
                    ChildState {
                        id,
                        entry,
                        respawn: true,
                    },
                );
            }
        }
        Ok(())
    }

    fn transition_runlevel(
        &mut self,
        new_rl: u8,
        inittab: &InitTab,
        rl_mgr: &mut RunlevelManager,
    ) -> Result<()> {
        info!("transitioning to runlevel {}", new_rl);
        let old = rl_mgr.current();
        rl_mgr.set(new_rl);

        // Kill processes not in new runlevel (simplified)
        // In full impl: track which entries are active

        // Run rc script
        let rc_cmd = format!("/etc/rc.d/rc {}", new_rl);
        let _ = process::spawn_async(&rc_cmd, false);

        // Start entries for new runlevel
        for entry in inittab.entries_for_runlevel(new_rl) {
            match entry.action {
                Action::Respawn | Action::Once | Action::Wait | Action::Boot | Action::BootWait => {
                    if let Ok(pid) = process::spawn_async(&entry.process, matches!(entry.action, Action::Respawn)) {
                        self.children.insert(
                            pid,
                            ChildState {
                                id: entry.id.clone(),
                                entry: entry.clone(),
                                respawn: matches!(entry.action, Action::Respawn),
                            },
                        );
                    }
                }
                Action::OnDemand => {
                    // Special handling
                }
                _ => {}
            }
        }

        // Run rc.local if multi-user
        if (2..=5).contains(&new_rl) {
            let _ = process::spawn_async("/etc/rc.local", false);
        }

        Ok(())
    }

    fn check_control_file(&mut self, rl_mgr: &mut RunlevelManager, inittab: &InitTab) -> Result<()> {
        let path = "/run/azurewrath.ctl";
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(rl) = content.trim().parse::<u8>() {
                if rl <= 6 && rl != rl_mgr.current() {
                    self.transition_runlevel(rl, inittab, rl_mgr)?;
                }
            }
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }

    fn do_halt(&self) -> Result<()> {
        use crate::quotes::{log_azurewrath_quote_only, quote_for};
        log_azurewrath_quote_only(quote_for("outro"));
        info!("performing final halt");
        // Sync filesystems
        let _ = nix::unistd::sync();
        // Reboot or poweroff via syscall
        unsafe {
            libc::reboot(libc::RB_POWER_OFF);
        }
        Ok(())
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.epoll_fd);
        let _ = nix::unistd::close(self.signal_fd);
    }
}
