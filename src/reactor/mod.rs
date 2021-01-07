use std::io::{self, Read, Write};
use std::io::ErrorKind::WouldBlock;
use std::os::unix::io::AsRawFd;
use std::fmt;

use crate::{Interest, System, Result, Event};

mod combinators;
mod consumers;

pub use consumers::{FilterMap, Map};
pub use combinators::Chain;

pub type ReactorId = u64;

// -----------------------------------------------------------------------------
//     - Reaction -
// -----------------------------------------------------------------------------
pub enum Reaction<T> {
    Continue,
    Value(T),
    Event(crate::Event),
}

impl<T: std::fmt::Debug> std::fmt::Debug for Reaction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reaction::Continue => write!(f, "Continue"),
            Reaction::Value(val) => write!(f, "Value: {:?}", val),
            Reaction::Event(ev) => write!(f, "Event: {:?}", ev),
        }
    }
}

// -----------------------------------------------------------------------------
//     - Reactor -
// -----------------------------------------------------------------------------
pub trait Reactor : Sized {
    type Input;
    type Output;

    fn chain<T: Reactor<Input=Self::Output>>(self, second: T) -> Chain<Self, T> {
        Chain::new(self, second)
    }

    fn map<T, F>(self, f: F) -> Map<Self, F, T>
        where F: FnMut(Self::Output) -> T
    {
        Map::new(self, f)
    }

    fn filter_map<T, F>(self, f: F) -> FilterMap<Self, F, T>
        where F: FnMut(Self::Output) -> Option<T>
    {
        FilterMap::new(self, f)
    }

    fn react(&mut self, val: Reaction<Self::Input>) -> Reaction<Self::Output>;
}


// -----------------------------------------------------------------------------
//     - Poll Reactor -
// -----------------------------------------------------------------------------
pub struct PollReactor<T: AsRawFd> {
    inner: T,
    pub id: ReactorId,
    writable: bool,
    readable: bool,
}

impl<T: AsRawFd> PollReactor<T> {
    pub fn new(inner: T, interest: Interest) -> Result<Self> {
        let id = System::reserve();
        System::arm(&inner, interest, id)?;

        let instance = Self {
            inner,
            id,
            writable: false,
            readable: false,
        };

        Ok(instance)
    }

    pub fn rearm(&self, interest: Interest) -> Result<()>  {
        System::rearm(&self.inner, interest, self.id)?;
        Ok(())
    }

    pub fn update(&mut self, ev: &Event) {
        self.readable = ev.read;
        self.writable = ev.write;
    }

    pub fn readable(&self) -> bool {
        self.readable
    }

    pub fn writable(&self) -> bool {
        self.writable
    }
}

impl<T: AsRawFd> AsRawFd for PollReactor<T> {
    fn as_raw_fd(&self) -> i32 {
        self.inner.as_raw_fd()
    }

}

// -----------------------------------------------------------------------------
//     - Drop -
// -----------------------------------------------------------------------------
impl<T: AsRawFd> Drop for PollReactor<T> {
    fn drop(&mut self) {
        System::free(self.id);
    }
}

// -----------------------------------------------------------------------------
//     - AsRef -
// -----------------------------------------------------------------------------
impl<T: AsRawFd> AsRef<T> for PollReactor<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

// -----------------------------------------------------------------------------
//     - AsMut -
// -----------------------------------------------------------------------------
impl<T: AsRawFd> AsMut<T> for PollReactor<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

// -----------------------------------------------------------------------------
//     - Read -
// -----------------------------------------------------------------------------
impl<T: AsRawFd + Read> Read for PollReactor<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.as_mut().read(buf);

        match res {
            Ok(0) => self.readable = false,
            Ok(_) => {}
            Err(ref e) if e.kind() == WouldBlock => {
                self.readable = false;
                if self.writable {
                    self.rearm(Interest::ReadWrite);
                } else {
                    self.rearm(Interest::Read);
                }
            }
            Err(_) => self.readable = false,

        }

        res
    }
}

// -----------------------------------------------------------------------------
//     - Write -
// -----------------------------------------------------------------------------
impl<T: AsRawFd + Write> Write for PollReactor<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.as_mut().write(buf);
        match res {
            Err(ref e) if e.kind() == WouldBlock => {
                self.writable = false;
                self.rearm(Interest::Write);
            }
            Err(_) => self.writable = false,
            Ok(_) => {}
        }

        Ok(res?)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(self.as_mut().flush()?)
    }
}

// -----------------------------------------------------------------------------
//     - Debug -
// -----------------------------------------------------------------------------
impl<T: fmt::Debug + AsRawFd> fmt::Debug for PollReactor<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<PollReactor {:?} read: {}, write: {}>", self.inner, self.readable, self.writable)
    }
}
