use std::io::{self, Read, Write};
use std::time::Duration;

use super::System;
use crate::{res, Result, Interest};

pub struct Timer {
    pub fd: i32,
    pub reactor_id: u64,
}

impl Timer {
    pub fn new(expiration: Duration, interval: Option<Duration>) -> Result<Self> {
        let flags = libc::EFD_CLOEXEC | libc::EFD_NONBLOCK;
        let reactor_id = System::reserve();
        let clock_id = libc::CLOCK_MONOTONIC;
        let fd = res!(unsafe { libc::timerfd_create(clock_id, flags) });

        let interval = match interval {
            Some(d) => libc::timespec {
                tv_sec: d.as_secs() as i64,
                tv_nsec: d.subsec_nanos() as i64,
            },
            None => libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        };

        let new_value = libc::itimerspec {
            it_interval: interval,
            it_value: libc::timespec {
                tv_sec: expiration.as_secs() as i64,
                tv_nsec: expiration.subsec_nanos() as i64,
            },
        };

        let flags = 0;
        let _ = res!(unsafe {
            libc::timerfd_settime(
                fd,
                flags,
                &new_value as *const libc::itimerspec,
                std::ptr::null_mut(), // old_value
            )
        });

        System::arm(&fd, Interest::Read, reactor_id)?;

        let inst = Self { fd, reactor_id };
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
}

// -----------------------------------------------------------------------------
//     - Write -
// -----------------------------------------------------------------------------
impl Write for Timer {
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
impl Read for Timer {
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
