//! CLI tests and helpers

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_string() {
        let v = env!("CARGO_PKG_VERSION");
        assert!(!v.is_empty());
    }
}

/// Parse runlevel from string
pub fn parse_runlevel(s: &str) -> Option<u8> {
    s.parse().ok().filter(|&n| n <= 6)
}

/// Format service status line
pub fn format_status(name: &str, running: bool, pid: Option<u32>) -> String {
    match (running, pid) {
        (true, Some(p)) => format!("{}: active (pid {})", name, p),
        (true, None) => format!("{}: active", name),
        (false, _) => format!("{}: inactive", name),
    }
}
