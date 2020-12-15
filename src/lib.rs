pub mod net;
pub mod errors;

mod reactor;
mod system;
mod codecs;
mod signals;

pub use reactor::{Reaction, Reactor, PollReactor};
pub use system::{Interest, System};
pub use system::userfd::Evented;
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
            -1 => return Err(crate::errors::os_err()),
            val => val
        }
    }
}
