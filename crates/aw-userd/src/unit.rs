//! User service unit file parser (simple key=value)

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct UserUnit {
    pub description: String,
    pub exec_start: String,
    pub exec_stop: Option<String>,
    pub restart: String,
    pub wanted_by: Vec<String>,
}

pub fn parse_user_unit(path: &Path) -> Result<UserUnit> {
    let content = fs::read_to_string(path)?;
    let mut unit = UserUnit::default();
    let mut section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim();
            match (section.as_str(), k) {
                ("Unit", "Description") => unit.description = v.to_string(),
                ("Service", "ExecStart") => unit.exec_start = v.to_string(),
                ("Service", "ExecStop") => unit.exec_stop = Some(v.to_string()),
                ("Service", "Restart") => unit.restart = v.to_string(),
                ("Install", "WantedBy") => unit.wanted_by.push(v.to_string()),
                _ => {}
            }
        }
    }
    Ok(unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_simple() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "[Unit]").unwrap();
        writeln!(f, "Description=Test Service").unwrap();
        writeln!(f, "[Service]").unwrap();
        writeln!(f, "ExecStart=/bin/true").unwrap();
        writeln!(f, "Restart=on-failure").unwrap();

        let u = parse_user_unit(f.path()).unwrap();
        assert_eq!(u.description, "Test Service");
        assert_eq!(u.exec_start, "/bin/true");
        assert_eq!(u.restart, "on-failure");
    }
}
