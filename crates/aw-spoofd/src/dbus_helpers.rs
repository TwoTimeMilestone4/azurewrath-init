//! D-Bus message construction helpers and tests for spoofd

use zvariant::{OwnedValue, Value, Signature};

/// Build a typical ListUnits reply entry
pub fn make_unit_tuple(
    name: &str,
    desc: &str,
    load: &str,
    active: &str,
    sub: &str,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    u32,
    String,
    String,
) {
    (
        name.to_string(),
        desc.to_string(),
        load.to_string(),
        active.to_string(),
        sub.to_string(),
        String::new(),
        format!("/org/freedesktop/systemd1/unit/{}", name.replace('.', "_")),
        0,
        String::new(),
        "/".to_string(),
    )
}

/// Parse a simple sd_notify payload
pub fn parse_notify(payload: &str) -> Vec<(String, String)> {
    payload
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                _ => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_notify_ready() {
        let pairs = parse_notify("READY=1\nSTATUS=Running\nMAINPID=123");
        assert!(pairs.iter().any(|(k, v)| k == "READY" && v == "1"));
        assert!(pairs.iter().any(|(k, v)| k == "MAINPID" && v == "123"));
    }

    #[test]
    fn unit_tuple() {
        let t = make_unit_tuple("foo.service", "Foo", "loaded", "active", "running");
        assert_eq!(t.0, "foo.service");
        assert_eq!(t.3, "active");
    }
}
