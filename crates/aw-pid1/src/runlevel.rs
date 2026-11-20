//! Runlevel manager

use log::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunlevelManager {
    current: u8,
    previous: u8,
}

impl RunlevelManager {
    pub fn new(initial: u8) -> Self {
        RunlevelManager {
            current: initial,
            previous: initial,
        }
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn previous(&self) -> u8 {
        self.previous
    }

    pub fn set(&mut self, new: u8) {
        info!("runlevel {} -> {}", self.current, new);
        self.previous = self.current;
        self.current = new;
    }

    pub fn is_multiuser(&self) -> bool {
        (2..=5).contains(&self.current)
    }

    pub fn is_graphical(&self) -> bool {
        self.current == 5
    }

    pub fn is_halt(&self) -> bool {
        self.current == 0
    }

    pub fn is_reboot(&self) -> bool {
        self.current == 6
    }

    pub fn is_single(&self) -> bool {
        self.current == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions() {
        let mut rl = RunlevelManager::new(3);
        assert_eq!(rl.current(), 3);
        rl.set(5);
        assert_eq!(rl.current(), 5);
        assert_eq!(rl.previous(), 3);
        assert!(rl.is_graphical());
        rl.set(0);
        assert!(rl.is_halt());
    }
}
