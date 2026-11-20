//! coredump spoof for DrKonqi
//! Stores crashes in /var/lib/systemd/coredump/ and exposes org.freedesktop.Coredump

use log::info;
use std::fs;
use std::path::PathBuf;
use zbus::{interface, Result as ZResult};

pub struct CoredumpManager {
    dir: PathBuf,
}

impl CoredumpManager {
    pub fn new(dir: &str) -> Self {
        let path = PathBuf::from(dir);
        let _ = fs::create_dir_all(&path);
        CoredumpManager { dir: path }
    }

    pub fn store_crash(&self, pid: u32, comm: &str, data: &[u8]) -> Result<String, std::io::Error> {
        let name = format!("core.{}.{}.{}", comm, pid, std::process::id());
        let path = self.dir.join(&name);
        fs::write(&path, data)?;
        info!("stored coredump: {}", path.display());
        Ok(name)
    }

    pub fn list(&self) -> Vec<String> {
        fs::read_dir(&self.dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[interface(name = "org.freedesktop.Coredump")]
impl CoredumpManager {
    async fn get_metadata(&self, _pid: u32) -> ZResult<Vec<(String, String)>> {
        Ok(vec![
            ("COREDUMP_PID".into(), "0".into()),
            ("COREDUMP_COMM".into(), "unknown".into()),
        ])
    }
}
