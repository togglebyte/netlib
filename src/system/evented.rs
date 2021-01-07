use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};

use libc::eventfd;

use super::System;
use crate::{res, Interest, Result};

#[derive(Debug, Clone, Copy)]
pub struct Evented {
    pub fd: i32,
    pub reactor_id: u64,
}

impl Evented {
    pub fn new() -> Result<Self> {
        let flags = libc::EFD_CLOEXEC | libc::EFD_NONBLOCK;
        let fd = res!(unsafe { eventfd(0, flags) });
        let reactor_id = System::reserve();

        System::arm(&fd, Interest::Read, reactor_id)?;

        let inst = Self {
            fd,
            reactor_id,
        };

        Ok(inst)
    }

    pub fn consume_event(&mut self) -> Result<()> {
        let mut buf = [0u8; 8];
        let res = self.read(&mut buf)?;
        let res = self.rearm()?;
        Ok(())
    }

    fn rearm(&mut self) -> Result<()> {
        System::rearm(&self.fd, Interest::Read, self.reactor_id)
    }

    pub fn poke(&mut self) -> Result<()> {
        let val = 1u64.to_be_bytes();
        let _ = self.write(&val)?;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//     - AsRawFd -
// -----------------------------------------------------------------------------
impl AsRawFd for Evented {
    fn as_raw_fd(&self) -> i32 {
        self.fd
    }
}

// -----------------------------------------------------------------------------
//     - Write -
// -----------------------------------------------------------------------------
impl Write for Evented {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let p = buf.as_ptr() as *const libc::c_void;
        let len = buf.len();
        let res = unsafe { libc::write(self.fd, p, len) };
        match res {
            -1 => Err(crate::errors::os_err()),
            n => Ok(n as usize),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//     - Read -
// -----------------------------------------------------------------------------
impl Read for Evented {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let p = buf.as_mut_ptr() as *mut libc::c_void;
        let len = buf.len();
        let res = unsafe { libc::read(self.fd, p, len) };
        match res {
            -1 => Err(crate::errors::os_err()),
            n => Ok(n as usize),
        }
    }
}
