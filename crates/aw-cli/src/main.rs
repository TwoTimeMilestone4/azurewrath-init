//! azurewrath CLI
//! Mapping: SysV function + Azurewrath (Forsaken) skill name

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

mod helpers;
mod quotes;

use quotes::{log_azurewrath, log_azurewrath_quote_only, quote_for};

#[derive(Parser)]
#[command(name = "azurewrath", version, about = "azurewrath-init CLI - SysVinit-compatible")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enable service (create S symlink) - sow
    Sow { service: String },
    /// Disable service (remove S symlink) - uproot
    Uproot { service: String },
    /// Start service - bloom
    Bloom { service: String },
    /// Stop service - wilt
    Wilt { service: String },
    /// Restart service - regrow
    Regrow { service: String },
    /// Reboot system - revive
    Revive,
    /// Poweroff / halt - reap
    Reap,
    /// Show status
    Status { service: Option<String> },
    /// List all services
    List,
    /// Show logs
    Logs { service: Option<String> },
    /// Reload inittab
    Reload,
    /// Change runlevel
    Telinit { runlevel: u8 },
    /// Alias for telinit
    Runlevel { runlevel: Option<u8> },
    /// List sessions
    Session { action: String },
    /// List coredumps
    Coredump { action: String },
    /// Version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Sow { service } => cmd_sow(&service)?,
        Commands::Uproot { service } => cmd_uproot(&service)?,
        Commands::Bloom { service } => cmd_bloom(&service)?,
        Commands::Wilt { service } => cmd_wilt(&service)?,
        Commands::Regrow { service } => cmd_regrow(&service)?,
        Commands::Revive => cmd_revive()?,
        Commands::Reap => cmd_reap()?,
        Commands::Status { service } => cmd_status(service.as_deref())?,
        Commands::List => cmd_list()?,
        Commands::Logs { service } => cmd_logs(service.as_deref())?,
        Commands::Reload => cmd_reload()?,
        Commands::Telinit { runlevel } => cmd_telinit(runlevel)?,
        Commands::Runlevel { runlevel } => {
            if let Some(rl) = runlevel {
                cmd_telinit(rl)?;
            } else {
                cmd_show_runlevel()?;
            }
        }
        Commands::Session { action } => cmd_session(&action)?,
        Commands::Coredump { action } => cmd_coredump(&action)?,
        Commands::Version => {
            log_azurewrath_quote_only(quote_for("status"));
            println!("Azurewrath: azurewrath {}", env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}

fn cmd_sow(service: &str) -> Result<()> {
    log_azurewrath(&format!("Sow {}", service), quote_for("sow"));
    for rl in 2..=5 {
        let dir = format!("/etc/rc{}.d", rl);
        let _ = fs::create_dir_all(&dir);
        let link = format!("{}/S99{}", dir, service);
        let target = format!("../init.d/{}", service);
        let _ = std::os::unix::fs::symlink(&target, &link);
    }
    Ok(())
}

fn cmd_uproot(service: &str) -> Result<()> {
    log_azurewrath(&format!("Uproot {}", service), quote_for("uproot"));
    for rl in 0..=6 {
        let dir = format!("/etc/rc{}.d", rl);
        let path = Path::new(&dir);
        if path.exists() {
            if let Ok(entries) = fs::read_dir(path) {
                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.contains(service) && (name.starts_with('S') || name.starts_with('K')) {
                        let _ = fs::remove_file(e.path());
                    }
                }
            }
        }
    }
    Ok(())
}

fn cmd_bloom(service: &str) -> Result<()> {
    log_azurewrath(&format!("Bloom {}", service), quote_for("bloom"));
    let script = format!("/etc/init.d/{}", service);
    let status = Command::new(&script).arg("start").status();
    match status {
        Ok(s) if s.success() => {}
        _ => {
            log_azurewrath(&format!("Bloom {} failed", service), quote_for("fail"));
        }
    }
    Ok(())
}

fn cmd_wilt(service: &str) -> Result<()> {
    log_azurewrath(&format!("Wilt {}", service), quote_for("wilt"));
    let script = format!("/etc/init.d/{}", service);
    let _ = Command::new(&script).arg("stop").status();
    Ok(())
}

fn cmd_regrow(service: &str) -> Result<()> {
    log_azurewrath(&format!("Regrow {}", service), quote_for("regrow"));
    cmd_wilt(service)?;
    cmd_bloom(service)?;
    Ok(())
}

fn cmd_revive() -> Result<()> {
    log_azurewrath("Revive", quote_for("revive"));
    fs::write("/run/azurewrath.ctl", "6\n")?;
    Ok(())
}

fn cmd_reap() -> Result<()> {
    log_azurewrath("Reap", quote_for("reap"));
    fs::write("/run/azurewrath.ctl", "0\n")?;
    Ok(())
}

fn cmd_status(service: Option<&str>) -> Result<()> {
    log_azurewrath("Status", quote_for("status"));
    if let Some(svc) = service {
        println!("Azurewrath: status of {}", svc);
        let script = format!("/etc/init.d/{}", svc);
        let _ = Command::new(&script).arg("status").status();
    } else {
        println!("Azurewrath: runlevel: {}", current_runlevel());
        println!("Azurewrath: log: /run/azurewrath.log");
    }
    Ok(())
}

fn cmd_list() -> Result<()> {
    log_azurewrath_quote_only(quote_for("status"));
    println!("Azurewrath: services in /etc/init.d:");
    if let Ok(entries) = fs::read_dir("/etc/init.d") {
        for e in entries.flatten() {
            if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy().to_string();
                let enabled = is_enabled(&name);
                println!("Azurewrath:   {} [{}]", name, if enabled { "enabled" } else { "disabled" });
            }
        }
    }
    Ok(())
}

fn cmd_logs(_service: Option<&str>) -> Result<()> {
    log_azurewrath_quote_only(quote_for("status"));
    let log = "/run/azurewrath.log";
    if Path::new(log).exists() {
        let content = fs::read_to_string(log)?;
        println!("Azurewrath: {}", content);
    } else {
        println!("Azurewrath: no log yet");
    }
    Ok(())
}

fn cmd_reload() -> Result<()> {
    log_azurewrath("Reload", quote_for("regrow"));
    let _ = nix::sys::signal::kill(nix::unistd::Pid::from_raw(1), nix::sys::signal::Signal::SIGHUP);
    Ok(())
}

fn cmd_telinit(rl: u8) -> Result<()> {
    log_azurewrath(&format!("Telinit {}", rl), quote_for("regrow"));
    fs::write("/run/azurewrath.ctl", format!("{}\n", rl))?;
    Ok(())
}

fn cmd_show_runlevel() -> Result<()> {
    log_azurewrath("Runlevel", quote_for("status"));
    println!("Azurewrath: {}", current_runlevel());
    Ok(())
}

fn cmd_session(action: &str) -> Result<()> {
    log_azurewrath_quote_only(quote_for("status"));
    match action {
        "list" => {
            println!("Azurewrath: sessions (via spoofd):");
            println!("Azurewrath:   c1  1000  user  seat0");
        }
        _ => println!("Azurewrath: unknown session action"),
    }
    Ok(())
}

fn cmd_coredump(action: &str) -> Result<()> {
    log_azurewrath_quote_only(quote_for("status"));
    match action {
        "list" => {
            println!("Azurewrath: coredumps:");
            if let Ok(entries) = fs::read_dir("/var/lib/systemd/coredump") {
                for e in entries.flatten() {
                    println!("Azurewrath:   {}", e.file_name().to_string_lossy());
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_enabled(service: &str) -> bool {
    for rl in 2..=5 {
        let dir = format!("/etc/rc{}.d", rl);
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('S') && name.contains(service) {
                    return true;
                }
            }
        }
    }
    false
}

fn current_runlevel() -> String {
    fs::read_to_string("/run/azurewrath.runlevel")
        .unwrap_or_else(|_| "3".into())
        .trim()
        .to_string()
}
