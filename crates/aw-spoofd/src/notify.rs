//! sd_notify protocol listener
//! Listens on $NOTIFY_SOCKET (/run/systemd/notify)

use anyhow::Result;
use log::info;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use tokio::net::UnixDatagram as TokioUnixDatagram;

pub struct NotifyListener;

impl NotifyListener {
    pub async fn start(path: &str) -> Result<Self> {
        let p = Path::new(path);
        if p.exists() {
            let _ = std::fs::remove_file(p);
        }
        let sock = UnixDatagram::bind(path)?;
        info!("sd_notify listening on {}", path);

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match sock.recv(&mut buf) {
                    Ok(n) => {
                        let msg = String::from_utf8_lossy(&buf[..n]);
                        info!("sd_notify: {}", msg.trim());
                        // Parse READY=1, STATUS=, MAINPID=, WATCHDOG=1
                        // Return immediately - no blocking
                        if msg.contains("READY=1") {
                            info!("service ready notification received");
                        }
                        if msg.contains("WATCHDOG=1") {
                            // Acknowledge watchdog
                        }
                    }
                    Err(e) => {
                        log::warn!("notify recv error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(NotifyListener)
    }
}
