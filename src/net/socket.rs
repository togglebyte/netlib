use std::io::Result;
use std::net::SocketAddr;

use libc::{c_int, c_void, socklen_t, sockaddr};

use crate::res;

fn setsockopt<T>(sock: &Socket, opt: c_int, val: c_int, payload: T) -> Result<()> {
    unsafe {
        let payload = &payload as *const T as *const c_void;
        let _ = res!(libc::setsockopt(
            sock.as_inner(),
            opt,
            val,
            payload,
            std::mem::size_of::<T>() as socklen_t,
        ));
        Ok(())
    }
}

fn into_inner(addr: &SocketAddr) -> (*const sockaddr, socklen_t) {
    match *addr {
        SocketAddr::V4(ref a) => (
            a as *const _ as *const _,
            std::mem::size_of_val(a) as socklen_t,
        ),
        SocketAddr::V6(ref a) => (
            a as *const _ as *const _,
            std::mem::size_of_val(a) as socklen_t,
        ),
    }
}

pub(super) struct Socket(pub c_int);

impl Socket {
    pub fn new(addr: Result<&SocketAddr>) -> Result<Self> {
        let addr = addr?;
        let family = match addr {
            SocketAddr::V4(..) => libc::AF_INET,
            SocketAddr::V6(..) => libc::AF_INET6,
        };

        let (addr_ptr, addr_len) = into_inner(&addr);

        let socket = Self(unsafe { res!(libc::socket(family, libc::SOCK_STREAM, 0)) });
        setsockopt(&socket, libc::SOL_SOCKET, libc::SO_REUSEADDR, 1 as c_int)?;
        setsockopt(&socket, libc::SOL_SOCKET, libc::SO_REUSEPORT, 1 as c_int)?;

        let _ = res!(unsafe { libc::bind(socket.as_inner(), addr_ptr, addr_len) });
        let _ = res!(unsafe { libc::listen(socket.as_inner(), 128) });

        Ok(socket)
    }

    fn as_inner(&self) -> libc::c_int {
        self.0
    }
}
