use libc::__errno_location as errno_loc;

pub fn os_err() -> std::io::Error {
    let err_num = unsafe { *errno_loc() };
    std::io::Error::from_raw_os_error(err_num)
}
