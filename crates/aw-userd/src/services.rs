//! User service manager

use anyhow::Result;
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use xdg::BaseDirectories;

pub struct UserService {
    pub name: String,
    pub exec: String,
    pub child: Option<Child>,
}

pub struct UserServiceManager {
    services: HashMap<String, UserService>,
    user_dir: PathBuf,
    system_dir: PathBuf,
}

impl UserServiceManager {
    pub fn new() -> Result<Self> {
        let xdg = BaseDirectories::new()?;
        let user_dir = xdg.get_config_home().join("azurewrath/user");
        let system_dir = PathBuf::from("/etc/azurewrath/user");
        std::fs::create_dir_all(&user_dir).ok();

        Ok(UserServiceManager {
            services: HashMap::new(),
            user_dir,
            system_dir,
        })
    }

    pub fn load_autostart(&mut self) -> Result<()> {
        // XDG autostart for GNOME/Plasma
        let autostart_dirs = [
            self.user_dir.join("autostart"),
            PathBuf::from("/etc/xdg/autostart"),
            PathBuf::from("/usr/share/applications"),
        ];

        for dir in &autostart_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            // Parse Exec= from .desktop
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Some(exec_line) = content.lines().find(|l| l.starts_with("Exec=")) {
                                    let exec = exec_line.trim_start_matches("Exec=").to_string();
                                    info!("autostart: {} -> {}", name, exec);
                                    self.services.insert(
                                        name.to_string(),
                                        UserService {
                                            name: name.to_string(),
                                            exec,
                                            child: None,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Special for GNOME/Plasma
        self.services.insert(
            "gnome-shell".into(),
            UserService {
                name: "gnome-shell".into(),
                exec: "gnome-shell".into(),
                child: None,
            },
        );
        self.services.insert(
            "plasmashell".into(),
            UserService {
                name: "plasmashell".into(),
                exec: "plasmashell".into(),
                child: None,
            },
        );
        self.services.insert(
            "kwin_wayland".into(),
            UserService {
                name: "kwin_wayland".into(),
                exec: "kwin_wayland".into(),
                child: None,
            },
        );

        Ok(())
    }

    pub fn start(&mut self, name: &str) -> Result<()> {
        if let Some(svc) = self.services.get_mut(name) {
            if svc.child.is_none() {
                info!("starting user service: {}", name);
                let child = Command::new("sh")
                    .arg("-c")
                    .arg(&svc.exec)
                    .spawn()?;
                svc.child = Some(child);
            }
        }
        Ok(())
    }

    pub fn stop(&mut self, name: &str) -> Result<()> {
        if let Some(svc) = self.services.get_mut(name) {
            if let Some(mut child) = svc.child.take() {
                info!("stopping user service: {}", name);
                let _ = child.kill();
            }
        }
        Ok(())
    }

    pub fn run_event_loop(&mut self) -> Result<()> {
        info!("userd event loop running");
        // Non-blocking style: poll children
        loop {
            for svc in self.services.values_mut() {
                if let Some(child) = &mut svc.child {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            info!("{} exited: {}", svc.name, status);
                            svc.child = None;
                        }
                        Ok(None) => {}
                        Err(e) => log::warn!("wait error: {}", e),
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}
