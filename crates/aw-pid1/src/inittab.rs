//! inittab parser
//! Format: id:runlevels:action:process
//! Clean-room implementation.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Sysinit,
    Wait,
    Once,
    Respawn,
    CtrlAltDel,
    PowerFail,
    PowerOk,
    PowerFailNow,
    InitDefault,
    Boot,
    BootWait,
    Off,
    OnDemand,
    Shutdown,
    Unknown(String),
}

impl Action {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sysinit" => Action::Sysinit,
            "wait" => Action::Wait,
            "once" => Action::Once,
            "respawn" => Action::Respawn,
            "ctrlaltdel" => Action::CtrlAltDel,
            "powerfail" => Action::PowerFail,
            "powerok" => Action::PowerOk,
            "powerfailnow" => Action::PowerFailNow,
            "initdefault" => Action::InitDefault,
            "boot" => Action::Boot,
            "bootwait" => Action::BootWait,
            "off" => Action::Off,
            "ondemand" => Action::OnDemand,
            "shutdown" => Action::Shutdown,
            other => Action::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitEntry {
    pub id: String,
    pub runlevels: Vec<u8>,
    pub action: Action,
    pub process: String,
}

#[derive(Debug, Default)]
pub struct InitTab {
    entries: Vec<InitEntry>,
    by_id: HashMap<String, usize>,
}

impl InitTab {
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path).context("open inittab")?;
        let reader = BufReader::new(file);
        let mut tab = InitTab::default();

        for (lineno, line_res) in reader.lines().enumerate() {
            let line = line_res.context("read line")?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(entry) = parse_line(line, lineno + 1) {
                let idx = tab.entries.len();
                tab.by_id.insert(entry.id.clone(), idx);
                tab.entries.push(entry);
            }
        }
        Ok(tab)
    }

    pub fn initdefault(&self) -> Option<u8> {
        self.entries.iter().find_map(|e| {
            if matches!(e.action, Action::InitDefault) && !e.runlevels.is_empty() {
                Some(e.runlevels[0])
            } else {
                None
            }
        })
    }

    pub fn sysinit(&self) -> Option<&InitEntry> {
        self.entries.iter().find(|e| matches!(e.action, Action::Sysinit))
    }

    pub fn entries_for_runlevel(&self, rl: u8) -> Vec<&InitEntry> {
        self.entries
            .iter()
            .filter(|e| e.runlevels.is_empty() || e.runlevels.contains(&rl))
            .collect()
    }

    pub fn find_by_id(&self, id: &str) -> Option<&InitEntry> {
        self.by_id.get(id).map(|&i| &self.entries[i])
    }

    pub fn all_entries(&self) -> &[InitEntry] {
        &self.entries
    }

    pub fn reload(&mut self, path: &Path) -> Result<()> {
        *self = Self::load(path)?;
        Ok(())
    }
}

fn parse_line(line: &str, lineno: usize) -> Option<InitEntry> {
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() < 4 {
        log::warn!("inittab line {} malformed: {}", lineno, line);
        return None;
    }
    let id = parts[0].to_string();
    let rls_str = parts[1];
    let action = Action::parse(parts[2]);
    let process = parts[3].to_string();

    let runlevels = if rls_str.is_empty() {
        vec![]
    } else {
        rls_str
            .chars()
            .filter_map(|c| c.to_digit(10).map(|d| d as u8))
            .filter(|&d| d <= 6)
            .collect()
    };

    Some(InitEntry {
        id,
        runlevels,
        action,
        process,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_basic() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "# comment").unwrap();
        writeln!(f, "si::sysinit:/etc/rc.sysinit").unwrap();
        writeln!(f, "id:3:initdefault:").unwrap();
        writeln!(f, "1:2345:respawn:/sbin/getty 38400 tty1").unwrap();
        writeln!(f, "ca::ctrlaltdel:/sbin/shutdown -t3 -r now").unwrap();

        let tab = InitTab::load(f.path()).unwrap();
        assert_eq!(tab.initdefault(), Some(3));
        assert!(tab.sysinit().is_some());
        let respawn = tab.entries_for_runlevel(3);
        assert!(respawn.iter().any(|e| e.id == "1"));
    }
}
