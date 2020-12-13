use std::io::Result;
use crate::res;

pub struct SocketBuilder(libc::c_int);

impl SocketBuilder {
    pub fn new() -> Result<Self> {
        let fd = res!(libc::socket(family, libc::SOCK_STREAM, 0));
        Ok(Self(fd))
    }
}


pub struct Socket(pub libc::c_int);

// impl Socket {
//         ::cvt(c::bind(self.inner.raw(), addr.as_ptr(), len as c::socklen_t)).map(|_| ())
// }
