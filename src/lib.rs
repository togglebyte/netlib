pub mod net;
pub mod signals;

mod errors;
mod reactor;
mod system;
mod codecs;

pub use reactor::{Reaction, Reactor, PollReactor};
pub use system::{Interest, System};
pub use system::evented::Evented;
pub use errors::{Error, Result};

#[derive(Debug)]
pub struct Event {
    pub read: bool,
    pub write: bool,
    pub owner: u64,
}

#[macro_export]
macro_rules! res {
    ($e:expr) => {
        match $e {
            -1 => return Err(crate::errors::Error::Io(crate::errors::os_err().into())),
            val => val
        }
    }
}
