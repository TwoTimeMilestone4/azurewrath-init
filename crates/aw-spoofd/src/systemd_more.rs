//! More systemd Manager methods that desktop environments probe

use super::SystemdManager;
use zbus::interface;
use zvariant::{OwnedObjectPath, Value};

impl SystemdManager {
    /// List jobs
    pub async fn list_jobs(&self) -> Vec<(u32, String, String, String, OwnedObjectPath, OwnedObjectPath)> {
        vec![]
    }

    /// Get job
    pub async fn get_job(&self, id: u32) -> OwnedObjectPath {
        OwnedObjectPath::try_from(format!("/org/freedesktop/systemd1/job/{}", id)).unwrap()
    }

    /// Cancel job
    pub async fn cancel_job(&self, _id: u32) -> zbus::Result<()> {
        Ok(())
    }

    /// Clear jobs
    pub async fn clear_jobs(&self) -> zbus::Result<()> {
        Ok(())
    }

    /// Reset failed
    pub async fn reset_failed(&self) -> zbus::Result<()> {
        Ok(())
    }

    /// Reload daemon
    pub async fn reload(&self) -> zbus::Result<()> {
        info!("daemon-reload requested");
        Ok(())
    }

    /// Reexecute
    pub async fn reexecute(&self) -> zbus::Result<()> {
        Ok(())
    }
}

use log::info;
