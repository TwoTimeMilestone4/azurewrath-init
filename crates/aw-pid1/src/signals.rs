//! Signal handling helpers

use anyhow::{Context, Result};
use nix::sys::signalfd::siginfo;
use nix::sys::signal::Signal;
use std::os::unix::io::RawFd;

pub struct SignalHandler {
    // reserved for future state
}

impl SignalHandler {
    pub fn new() -> Result<Self> {
        Ok(SignalHandler {})
    }

    pub fn read_signal(&mut self, fd: RawFd) -> Result<Option<Signal>> {
        let mut info = siginfo::default();
        let n = unsafe {
            libc::read(
                fd,
                &mut info as *mut _ as *mut libc::c_void,
                std::mem::size_of::<siginfo>() as libc::size_t,
            )
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(None);
            }
            return Err(err).context("read signalfd");
        }
        if n == 0 {
            return Ok(None);
        }
        // signo is in the first field
        let signo = unsafe { info.ssi_signo };
        Signal::try_from(signo as i32).ok().map(Some).transpose()
            .map_err(|e| anyhow::anyhow!("invalid signal: {}", e))
    }
}
