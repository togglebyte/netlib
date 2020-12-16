use std::io::Error as IoError;

use crossbeam::channel::TryRecvError;
use libc::__errno_location as errno_loc;

pub fn os_err() -> std::io::Error {
    let err_num = unsafe { *errno_loc() };
    std::io::Error::from_raw_os_error(err_num)
}

pub type Result<T> = std::result::Result<T, Error>;

// -----------------------------------------------------------------------------
//     - Erro -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Recv(TryRecvError)
}

// -----------------------------------------------------------------------------
//     - IO -
// -----------------------------------------------------------------------------
impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

// -----------------------------------------------------------------------------
//     - Cross beam -
// -----------------------------------------------------------------------------
impl From<TryRecvError> for Error {
    fn from(e: TryRecvError) -> Self {
        Error::Recv(e)
    }
}
