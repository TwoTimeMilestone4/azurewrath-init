//! systemctl shim
//! Translates systemctl commands to azurewrath equivalents

use anyhow::Result;
use std::env;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("systemctl: missing command");
        std::process::exit(1);
    }

    let is_user = args.iter().any(|a| a == "--user");
    let filtered: Vec<&str> = args.iter().filter(|a| *a != "--user").map(|s| s.as_str()).collect();

    if is_user {
        // Forward to aw-userd
        let mut cmd = Command::new("aw-userd");
        cmd.args(&filtered);
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    let cmd = filtered.first().copied().unwrap_or("");
    let service = filtered.get(1).copied();

    let azure_cmd = match cmd {
        "start" => "bloom",
        "stop" => "wilt",
        "restart" => "regrow",
        "enable" => "sow",
        "disable" => "uproot",
        "status" => "status",
        "is-active" => "status",
        "is-enabled" => "status",
        "daemon-reload" => "reload",
        "poweroff" | "halt" => "reap",
        "reboot" => "revive",
        "list-units" | "list-unit-files" => "list",
        _ => {
            eprintln!("systemctl: unsupported command: {}", cmd);
            std::process::exit(1);
        }
    };

    let mut azure = Command::new("azurewrath");
    azure.arg(azure_cmd);
    if let Some(svc) = service {
        azure.arg(svc);
    }
    let status = azure.status()?;
    std::process::exit(status.code().unwrap_or(0));
}
