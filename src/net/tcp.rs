use std::convert::TryFrom;
use std::io::Result;
use std::net::{
    Shutdown, SocketAddr, TcpListener as StdTcpListener, TcpStream as StdTcpStream, ToSocketAddrs,
};


use crate::{Interest, PollReactor, Reaction, Reactor};

// -----------------------------------------------------------------------------
//     - TcpListener -
// -----------------------------------------------------------------------------
pub type TcpListener = PollReactor<StdTcpListener>;

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        // let listener = StdTcpListener::bind(addr)?;
        // listener.set_nonblocking(true)?;
        // Self::new(listener, Interest::Read)


        let addr = match addr.to_socket_addrs()?.next() {
            Some(a) => a,
            None => panic!("TODO: this should be an io error"),
        };
        
        // for a in addr.to_socket_addrs() {
        // }

        use net2::unix::*;
        let mut listener = net2::TcpBuilder::new_v4().unwrap();
        listener.reuse_port(true);
        listener.bind(addr);

        let listener = listener.listen(1024).unwrap();
        listener.set_nonblocking(true);
        Self::new(listener, Interest::Read)
    }
}

impl Reactor for TcpListener {
    type Input = ();
    type Output = Result<(StdTcpStream, SocketAddr)>;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if ev.owner != self.id => Reaction::Event(ev),
            _ => {
                match self.rearm(Interest::Read) {
                    Ok(_) => Reaction::Value(self.as_mut().accept()),
                    Err(e) => Reaction::Value(Err(e)),
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
//     - TcpStream -
// -----------------------------------------------------------------------------
pub type TcpStream = PollReactor<StdTcpStream>;

impl TcpStream {
    pub fn close(&mut self) -> Result<()> {
        self.as_mut().shutdown(Shutdown::Both)
    }
}

impl TryFrom<StdTcpStream> for TcpStream {
    type Error = std::io::Error;

    fn try_from(s: StdTcpStream) -> Result<Self> {
        TcpStream::new(s, Interest::Read)
    }
}
