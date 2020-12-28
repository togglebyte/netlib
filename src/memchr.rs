use libc::c_void;

pub fn memchr(slice: &[u8], b: u8) -> Option<usize> {
    let p = slice.as_ptr();

    let res = unsafe { libc::memchr(p as *const c_void, b as i32, slice.len()) as *mut usize };

    match res.is_null() {
        false => Some(res as usize - p as usize),
        true => None,
    }
}
