//! Additional D-Bus property and method implementations for spoofd
//! Covers more of the interfaces GNOME and PLM probe.

use super::SystemdManager;
use zbus::{interface, Result as ZResult};
use zvariant::OwnedObjectPath;

impl SystemdManager {
    /// Get environment of the manager
    pub async fn environment(&self) -> Vec<String> {
        std::env::vars()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect()
    }

    /// Architecture
    pub async fn architecture(&self) -> String {
        std::env::consts::ARCH.to_string()
    }

    /// Virtualization
    pub async fn virtualization(&self) -> String {
        "none".into()
    }

    /// Confirm shutdown
    pub async fn confirm_shutdown(&self) -> bool {
        false
    }
}

/// Unit object interface (minimal)
pub struct UnitObject {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
}

#[interface(name = "org.freedesktop.systemd1.Unit")]
impl UnitObject {
    #[zbus(property)]
    async fn id(&self) -> String {
        self.name.clone()
    }

    #[zbus(property)]
    async fn description(&self) -> String {
        self.description.clone()
    }

    #[zbus(property)]
    async fn load_state(&self) -> String {
        self.load_state.clone()
    }

    #[zbus(property)]
    async fn active_state(&self) -> String {
        self.active_state.clone()
    }

    #[zbus(property)]
    async fn sub_state(&self) -> String {
        self.sub_state.clone()
    }

    #[zbus(property)]
    async fn fragment_path(&self) -> String {
        format!("/etc/systemd/system/{}.service", self.name)
    }

    #[zbus(property)]
    async fn unit_file_state(&self) -> String {
        "enabled".into()
    }

    #[zbus(property)]
    async fn unit_file_preset(&self) -> String {
        "enabled".into()
    }
}

/// Job object (minimal)
pub struct JobObject {
    pub id: u32,
    pub unit: String,
    pub job_type: String,
    pub state: String,
}

#[interface(name = "org.freedesktop.systemd1.Job")]
impl JobObject {
    #[zbus(property)]
    async fn id(&self) -> u32 {
        self.id
    }

    #[zbus(property)]
    async fn unit(&self) -> String {
        self.unit.clone()
    }

    #[zbus(property)]
    async fn job_type(&self) -> String {
        self.job_type.clone()
    }

    #[zbus(property)]
    async fn state(&self) -> String {
        self.state.clone()
    }
}
