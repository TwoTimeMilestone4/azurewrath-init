//! Non-blocking logging to /dev/console and /run/azurewrath.log

use anyhow::Result;
use log::{LevelFilter, Log, Metadata, Record};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::Mutex;

pub struct Logger {
    file: Mutex<std::fs::File>,
    console: Mutex<std::fs::File>,
}

impl Logger {
    pub fn init(log_path: &str, console_path: &str) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        let console = OpenOptions::new()
            .write(true)
            .open(console_path)
            .unwrap_or_else(|_| OpenOptions::new().write(true).open("/dev/null").unwrap());

        let logger = Logger {
            file: Mutex::new(file),
            console: Mutex::new(console),
        };
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Info);
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= LevelFilter::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("[{}] {}\n", record.level(), record.args());
            if let Ok(mut f) = self.file.lock() {
                let _ = f.write_all(msg.as_bytes());
            }
            if let Ok(mut c) = self.console.lock() {
                let _ = c.write_all(msg.as_bytes());
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut f) = self.file.lock() {
            let _ = f.flush();
        }
        if let Ok(mut c) = self.console.lock() {
            let _ = c.flush();
        }
    }
}
