//! org.freedesktop.systemd1.Manager spoof

use log::{info, warn};
use std::collections::HashMap;
use zbus::{interface, Connection, Result as ZResult};
use zvariant::{OwnedObjectPath, OwnedValue, Value};

pub struct SystemdManager {
    units: HashMap<String, UnitState>,
    default_target: String,
}

#[derive(Clone, Debug)]
struct UnitState {
    active: bool,
    load_state: String,
    active_state: String,
    sub_state: String,
    description: String,
}

impl SystemdManager {
    pub fn new() -> Self {
        let mut units = HashMap::new();
        // Pre-seed some units that GNOME/PLM expect
        units.insert(
            "plasmalogin.service".to_string(),
            UnitState {
                active: false,
                load_state: "loaded".into(),
                active_state: "inactive".into(),
                sub_state: "dead".into(),
                description: "Plasma Login Manager".into(),
            },
        );
        units.insert(
            "gdm.service".to_string(),
            UnitState {
                active: false,
                load_state: "loaded".into(),
                active_state: "inactive".into(),
                sub_state: "dead".into(),
                description: "GNOME Display Manager".into(),
            },
        );
        units.insert(
            "dbus.service".to_string(),
            UnitState {
                active: true,
                load_state: "loaded".into(),
                active_state: "active".into(),
                sub_state: "running".into(),
                description: "D-Bus System Message Bus".into(),
            },
        );

        SystemdManager {
            units,
            default_target: "graphical.target".into(),
        }
    }

    fn map_to_sysv(&self, unit: &str) -> Option<String> {
        match unit {
            "plasmalogin.service" => Some("/etc/init.d/plasmalogin".into()),
            "gdm.service" => Some("/etc/init.d/gdm".into()),
            "sddm.service" => Some("/etc/init.d/sddm".into()),
            "lightdm.service" => Some("/etc/init.d/lightdm".into()),
            _ => None,
        }
    }
}

#[interface(name = "org.freedesktop.systemd1.Manager")]
impl SystemdManager {
    async fn start_unit(&mut self, name: &str, _mode: &str) -> ZResult<OwnedObjectPath> {
        info!("StartUnit: {}", name);
        if let Some(script) = self.map_to_sysv(name) {
            info!("mapping {} -> {}", name, script);
            // Special tty1 handling for display managers
            if name.contains("plasmalogin") || name.contains("gdm") || name.contains("sddm") {
                // Stop getty on tty1 if running
                let _ = std::process::Command::new("pkill")
                    .args(["-f", "getty.*tty1"])
                    .status();
            }
            let _ = std::process::Command::new(&script).arg("start").status();
        }
        if let Some(u) = self.units.get_mut(name) {
            u.active = true;
            u.active_state = "active".into();
            u.sub_state = "running".into();
        }
        Ok(OwnedObjectPath::try_from("/org/freedesktop/systemd1/job/1").unwrap())
    }

    async fn stop_unit(&mut self, name: &str, _mode: &str) -> ZResult<OwnedObjectPath> {
        info!("StopUnit: {}", name);
        if let Some(script) = self.map_to_sysv(name) {
            let _ = std::process::Command::new(&script).arg("stop").status();
            // Restart getty after DM exits
            if name.contains("plasmalogin") || name.contains("gdm") {
                let _ = std::process::Command::new("/sbin/getty")
                    .args(["38400", "tty1"])
                    .spawn();
            }
        }
        if let Some(u) = self.units.get_mut(name) {
            u.active = false;
            u.active_state = "inactive".into();
            u.sub_state = "dead".into();
        }
        Ok(OwnedObjectPath::try_from("/org/freedesktop/systemd1/job/2").unwrap())
    }

    async fn restart_unit(&mut self, name: &str, mode: &str) -> ZResult<OwnedObjectPath> {
        self.stop_unit(name, mode).await?;
        self.start_unit(name, mode).await
    }

    async fn reload_unit(&self, name: &str, _mode: &str) -> ZResult<OwnedObjectPath> {
        info!("ReloadUnit: {}", name);
        Ok(OwnedObjectPath::try_from("/org/freedesktop/systemd1/job/3").unwrap())
    }

    async fn get_unit(&self, name: &str) -> ZResult<OwnedObjectPath> {
        let path = format!("/org/freedesktop/systemd1/unit/{}", name.replace('.', "_"));
        Ok(OwnedObjectPath::try_from(path).unwrap())
    }

    async fn list_units(&self) -> ZResult<Vec<(String, String, String, String, String, String, OwnedObjectPath, u32, String, OwnedObjectPath)>> {
        let mut out = Vec::new();
        for (name, state) in &self.units {
            out.push((
                name.clone(),
                state.description.clone(),
                state.load_state.clone(),
                state.active_state.clone(),
                state.sub_state.clone(),
                String::new(),
                OwnedObjectPath::try_from("/org/freedesktop/systemd1/unit/dummy").unwrap(),
                0,
                String::new(),
                OwnedObjectPath::try_from("/").unwrap(),
            ));
        }
        Ok(out)
    }

    async fn list_unit_files(&self) -> ZResult<Vec<(String, String)>> {
        Ok(vec![
            ("plasmalogin.service".into(), "enabled".into()),
            ("gdm.service".into(), "disabled".into()),
        ])
    }

    async fn get_unit_file_state(&self, name: &str) -> ZResult<String> {
        Ok(if self.units.contains_key(name) {
            "enabled".into()
        } else {
            "disabled".into()
        })
    }

    async fn enable_unit_files(&self, _files: Vec<String>, _runtime: bool, _force: bool) -> ZResult<(bool, Vec<(String, String, String)>)> {
        Ok((true, vec![]))
    }

    async fn disable_unit_files(&self, _files: Vec<String>, _runtime: bool) -> ZResult<Vec<(String, String, String)>> {
        Ok(vec![])
    }

    async fn subscribe(&self) -> ZResult<()> {
        Ok(())
    }

    #[zbus(property)]
    async fn default_target(&self) -> String {
        self.default_target.clone()
    }

    async fn get_default_target(&self) -> ZResult<String> {
        Ok(self.default_target.clone())
    }

    async fn set_default_target(&mut self, name: &str, _force: bool) -> ZResult<Vec<(String, String, String)>> {
        self.default_target = name.to_string();
        Ok(vec![])
    }

    // Properties interface support is handled by zbus automatically for #[zbus(property)]
}

// Additional properties that GNOME checks
impl SystemdManager {
    pub async fn version(&self) -> String {
        "255.azurewrath".into()
    }

    pub async fn features(&self) -> String {
        "+PAM +AUDIT -SELINUX -IMA -APPARMOR +SMACK -SYSVINIT +UTMP +LIBCRYPTSETUP +GCRYPT +GNUTLS +ACL +XZ +LZ4 +ZSTD +SECCOMP +BLKID +ELFUTILS +KMOD +IDN2 -IDN +PCRE2 default-hierarchy=unified".into()
    }
}
