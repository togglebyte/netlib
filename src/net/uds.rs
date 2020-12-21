use std::convert::TryFrom;
use std::path::Path;
use std::net::Shutdown;
use std::os::unix::net::{UnixStream as StdUnixStream, UnixListener as StdUnixListener, SocketAddr};
use std::os::unix::io::FromRawFd;

use super::socket::Socket;
use crate::{Interest, PollReactor, Reaction, Reactor, Result};

// -----------------------------------------------------------------------------
//     - UnixListener -
// -----------------------------------------------------------------------------
pub type UnixListener = PollReactor<StdUnixListener>;

impl UnixListener {
    pub fn bind<P: AsRef<Path>>(path: P) -> Result<Self> {
        let listener = StdUnixListener::bind(path)?;
        listener.set_nonblocking(true)?;
        Self::new(listener, Interest::Read)
    }
}

impl Reactor for UnixListener {
    type Input = ();
    type Output = Result<(StdUnixStream, SocketAddr)>;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if ev.owner != self.id => Reaction::Event(ev),
            Reaction::Event(ev) if ev.read => {
                match self.rearm(Interest::Read) {
                    Err(e) => Reaction::Value(Err(e)),
                    Ok(_) => {
                        let val = match self.as_mut().accept() {
                            Err(e) => Err(crate::Error::Io(e)),
                            Ok(s) => Ok(s)
                        };
                        Reaction::Value(val)
                    }
                }
            }
            _ => Reaction::Continue,
        }
    }
}

// -----------------------------------------------------------------------------
//     - UnixStream -
// -----------------------------------------------------------------------------
pub type UnixStream = PollReactor<StdUnixStream>;

impl UnixStream {
    pub fn close(&mut self) -> Result<()> {
        self.as_mut().shutdown(Shutdown::Both)?;
        Ok(())
    }
}

impl TryFrom<StdUnixStream> for UnixStream {
    type Error = crate::Error;

    fn try_from(s: StdUnixStream) -> Result<Self> {
        UnixStream::new(s, Interest::Read)
    }
}

