use std::io::{self, Write, Read};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::File;

use libc::eventfd;

use crate::{Interest, res, Result};
use super::System;

#[derive(Debug, Copy, Clone)]
pub struct Evented {
    fd: i32,
    pub reactor_id: u64,
}

impl Evented {
    pub fn new() -> Result<Self> {
        let flags = libc::EFD_CLOEXEC | libc::EFD_NONBLOCK;// | libc::EFD_SEMAPHORE;
        let fd = res!(unsafe { eventfd(0, flags) });
        let reactor_id = System::reserve();

        System::arm(&fd, Interest::Read, reactor_id)?;

        let inst = Self {
            fd,
            reactor_id
        };

        Ok(inst)
    }

    pub fn rearm(&self) -> Result<()> {
        System::rearm(&self.fd, Interest::Read, self.reactor_id)
    }

    pub fn poke(&mut self) -> Result<()> {
        let val = 1u64.to_be_bytes();
        self.write(&val)?;
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
        let mut file = unsafe { File::from_raw_fd(self.fd) };
        file.write(buf)
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
        let mut file = unsafe { File::from_raw_fd(self.fd) };
        file.read(buf)
    }
}
