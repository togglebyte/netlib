#![allow(warnings)]
pub mod net;
pub mod broadcast;
pub mod queue;
pub mod memchr;

mod errors;
mod reactor;
mod system;
mod codecs;

pub use reactor::{Reaction, Reactor, PollReactor};
pub use system::{Interest, System, SysEvent};
pub use system::evented::Evented;
pub use system::timer::Timer;
pub use errors::{Error, Result, os_err};

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub read: bool,
    pub write: bool,
    pub owner: u64,
}

#[macro_export]
macro_rules! res {
    ($e:expr) => {
        match $e {
            -1 => return Err(crate::Error::Io(crate::os_err())),
            val => val
        }
    }
}
