//! userdb fallback to /etc/passwd

use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct UserDb {
    users: HashMap<u32, (String, String)>, // uid -> (name, home)
}

impl UserDb {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        if let Ok(file) = File::open("/etc/passwd") {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 6 {
                    if let Ok(uid) = parts[2].parse::<u32>() {
                        users.insert(uid, (parts[0].to_string(), parts[5].to_string()));
                    }
                }
            }
        }
        info!("userdb loaded {} users from /etc/passwd", users.len());
        UserDb { users }
    }

    pub fn lookup(&self, uid: u32) -> Option<&(String, String)> {
        self.users.get(&uid)
    }

    pub fn lookup_name(&self, name: &str) -> Option<u32> {
        self.users
            .iter()
            .find(|(_, (n, _))| n == name)
            .map(|(uid, _)| *uid)
    }
}
