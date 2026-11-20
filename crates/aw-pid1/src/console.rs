//! Console and early boot messaging
//! Non-blocking writes to /dev/console

use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

static CONSOLE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn init_console() -> Result<()> {
    let file = OpenOptions::new().write(true).open("/dev/console")?;
    *CONSOLE.lock().unwrap() = Some(file);
    Ok(())
}

pub fn console_write(msg: &str) {
    if let Ok(mut guard) = CONSOLE.lock() {
        if let Some(ref mut f) = *guard {
            let _ = writeln!(f, "{}", msg);
            let _ = f.flush();
        }
    }
}

pub fn console_info(msg: &str) {
    console_write(&format!("[azurewrath] {}", msg));
}

pub fn console_warn(msg: &str) {
    console_write(&format!("[azurewrath] WARNING: {}", msg));
}

pub fn console_error(msg: &str) {
    console_write(&format!("[azurewrath] ERROR: {}", msg));
}
