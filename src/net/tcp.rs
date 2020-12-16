use std::convert::TryFrom;
use std::net::{
    Shutdown, SocketAddr, TcpListener as StdTcpListener, TcpStream as StdTcpStream, ToSocketAddrs,
};
use std::os::unix::io::FromRawFd;

use super::socket::Socket;
use crate::{Interest, PollReactor, Reaction, Reactor, Result};

// -----------------------------------------------------------------------------
//     - TcpListener -
// -----------------------------------------------------------------------------
pub type TcpListener = PollReactor<StdTcpListener>;

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let addr = addr.to_socket_addrs()?.next().expect("Invalid address");
        let socket = Socket::new(Ok(&addr))?; // and this

        let listener = unsafe { StdTcpListener::from_raw_fd(socket.0) };
        listener.set_nonblocking(true)?;
        Self::new(listener, Interest::Read)
    }
}

impl Reactor for TcpListener {
    type Input = ();
    type Output = Result<(StdTcpStream, SocketAddr)>;

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
//     - TcpStream -
// -----------------------------------------------------------------------------
pub type TcpStream = PollReactor<StdTcpStream>;

impl TcpStream {
    pub fn close(&mut self) -> Result<()> {
        self.as_mut().shutdown(Shutdown::Both)?;
        Ok(())
    }
}

impl TryFrom<StdTcpStream> for TcpStream {
    type Error = crate::Error;

    fn try_from(s: StdTcpStream) -> Result<Self> {
        TcpStream::new(s, Interest::Read)
    }
}
