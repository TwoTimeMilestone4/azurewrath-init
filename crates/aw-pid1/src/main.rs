//! azurewrath-init PID 1
//!
//! SysVinit-compatible init written from scratch in Rust by TwoTimeMilestone4.
//! Non-blocking architecture: signalfd + epoll event loop.
//! Never blocks on service start, D-Bus, or child wait.

#![deny(unused_must_use)]
#![warn(clippy::all)]

use anyhow::{Context, Result};
use log::{error, info, warn};
use nix::sys::signal::{self, Signal};
use nix::unistd::{getpid, Pid};
use std::env;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

mod event_loop;
mod inittab;
mod inittab_ext;
mod logging;
mod mount;
mod process;
mod runlevel;
mod signals;
mod supervisor;
mod rc;
mod console;
mod helpers;
mod quotes;

use event_loop::EventLoop;
use inittab::InitTab;
use logging::Logger;
use mount::early_mounts;
use runlevel::RunlevelManager;
use signals::SignalHandler;
use quotes::{log_azurewrath_quote_only, quote_for};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    // If not PID 1, we may be invoked as halt/reboot/poweroff/shutdown
    let pid = getpid();
    if pid.as_raw() != 1 {
        return handle_non_pid1_invocation();
    }

    // We are PID 1
    log_azurewrath_quote_only(quote_for("boot"));
    info!("azurewrath-init {} starting as PID 1", VERSION);

    // Early setup
    early_mounts().context("early mounts failed")?;
    Logger::init("/run/azurewrath.log", "/dev/console")?;

    log_azurewrath_quote_only("Azurewrath-init PID 1 ready");

    // Load inittab
    let inittab_path = Path::new("/etc/inittab");
    let mut inittab = InitTab::load(inittab_path).context("failed to load /etc/inittab")?;

    // Determine initial runlevel
    let default_rl = inittab.initdefault().unwrap_or(3);
    info!("default runlevel from inittab: {}", default_rl);

    let mut rl_mgr = RunlevelManager::new(default_rl);
    let mut sig_handler = SignalHandler::new()?;
    let mut event_loop = EventLoop::new()?;

    // Initial sysinit
    if let Some(sysinit) = inittab.sysinit() {
        info!("running sysinit: {:?}", sysinit.process);
        process::spawn_async(&sysinit.process, false)?;
    }

    // Enter main event loop
    event_loop.run(&mut inittab, &mut rl_mgr, &mut sig_handler)?;

    Ok(())
}

fn handle_non_pid1_invocation() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let prog = Path::new(&args[0])
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("aw-pid1");

    match prog {
        "halt" | "poweroff" => {
            info!("requested poweroff");
            // Signal PID 1 via /run/azurewrath.ctl or direct
            request_runlevel(0)?;
        }
        "reboot" => {
            info!("requested reboot");
            request_runlevel(6)?;
        }
        "shutdown" => {
            // Parse simple flags
            let reboot = args.iter().any(|a| a == "-r" || a == "--reboot");
            let halt = args.iter().any(|a| a == "-h" || a == "--halt");
            if reboot {
                request_runlevel(6)?;
            } else if halt {
                request_runlevel(0)?;
            } else {
                request_runlevel(0)?;
            }
        }
        _ => {
            eprintln!("azurewrath-init: not PID 1 and unknown invocation: {}", prog);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn request_runlevel(rl: u8) -> Result<()> {
    // Write to control socket / FIFO that PID 1 monitors
    let path = "/run/azurewrath.ctl";
    std::fs::write(path, format!("{}\n", rl)).context("failed to write runlevel request")?;
    // Also send SIGHUP or custom signal if needed
    let _ = signal::kill(Pid::from_raw(1), Signal::SIGHUP);
    Ok(())
}
