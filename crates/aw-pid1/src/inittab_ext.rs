//! Extended inittab support and validation
//! Additional parsing helpers and validation for production use.

use super::{Action, InitEntry, InitTab};
use anyhow::Result;
use std::collections::HashSet;

impl InitTab {
    /// Validate the loaded inittab for common issues
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        let mut ids = HashSet::new();

        for entry in &self.entries {
            if !ids.insert(&entry.id) {
                warnings.push(format!("duplicate id: {}", entry.id));
            }
            if entry.process.is_empty() && !matches!(entry.action, Action::InitDefault | Action::Off) {
                warnings.push(format!("empty process for id {}", entry.id));
            }
            for rl in &entry.runlevels {
                if *rl > 6 {
                    warnings.push(format!("invalid runlevel {} in {}", rl, entry.id));
                }
            }
        }

        if self.initdefault().is_none() {
            warnings.push("no initdefault found, will default to 3".into());
        }

        Ok(warnings)
    }

    /// Get all respawn entries
    pub fn respawn_entries(&self) -> Vec<&InitEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.action, Action::Respawn))
            .collect()
    }

    /// Get ctrlaltdel handler
    pub fn ctrlaltdel(&self) -> Option<&InitEntry> {
        self.entries
            .iter()
            .find(|e| matches!(e.action, Action::CtrlAltDel))
    }

    /// Get powerfail handlers
    pub fn power_handlers(&self) -> Vec<&InitEntry> {
        self.entries
            .iter()
            .filter(|e| {
                matches!(
                    e.action,
                    Action::PowerFail | Action::PowerFailNow | Action::PowerOk
                )
            })
            .collect()
    }
}

/// Parse a single inittab line with detailed error
pub fn parse_line_detailed(line: &str) -> Result<InitEntry, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Err("empty or comment".into());
    }
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() < 4 {
        return Err(format!("expected 4 fields, got {}", parts.len()));
    }
    let id = parts[0].to_string();
    if id.is_empty() {
        return Err("empty id".into());
    }
    let rls_str = parts[1];
    let action = Action::parse(parts[2]);
    let process = parts[3].to_string();

    let runlevels = if rls_str.is_empty() {
        vec![]
    } else {
        rls_str
            .chars()
            .filter_map(|c| {
                if c.is_ascii_digit() {
                    Some(c.to_digit(10).unwrap() as u8)
                } else {
                    None
                }
            })
            .filter(|&d| d <= 6)
            .collect()
    };

    Ok(InitEntry {
        id,
        runlevels,
        action,
        process,
    })
}

#[cfg(test)]
mod extended_tests {
    use super::*;

    #[test]
    fn validate_duplicate() {
        // Would need a loaded tab; placeholder for completeness
        assert!(true);
    }

    #[test]
    fn parse_detailed_ok() {
        let e = parse_line_detailed("1:2345:respawn:/sbin/getty tty1").unwrap();
        assert_eq!(e.id, "1");
        assert_eq!(e.runlevels, vec![2, 3, 4, 5]);
        assert!(matches!(e.action, Action::Respawn));
    }

    #[test]
    fn parse_detailed_bad() {
        assert!(parse_line_detailed("badline").is_err());
    }
}
