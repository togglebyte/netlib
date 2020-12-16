use crate::{res, Result};

// -----------------------------------------------------------------------------
//     - Epoll abstraction -
//     * epoll_wait         [ ]
//     * epoll_ctl          [x]
//     * epoll_event        [?]
//     * epoll_create       [x]
//     * close              [x]
//     * flags              [x]
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum Interest {
    Read,
    Write,
    ReadWrite,
}

impl Interest {
    fn to_u32(self) -> u32 {
        match self {
            Interest::Read => Flags::Read as u32 | Flags::RHup as u32,
            Interest::Write => Flags::Write as u32,
            Interest::ReadWrite => Flags::Read as u32 | Flags::Write as u32 | Flags::RHup as u32,
        }
    }
}


// -----------------------------------------------------------------------------
//     - Create / Close -
// -----------------------------------------------------------------------------
pub(crate) fn create() -> Result<i32> {
    let fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    Ok(res!(fd))
}

pub fn close(epoll_fd: i32) -> Result<()> {
    unsafe { res!(libc::close(epoll_fd)) };
    Ok(())
}


// -----------------------------------------------------------------------------
//     - Epoll control -
// -----------------------------------------------------------------------------
pub(crate) fn arm(epoll_fd: i32, fd: i32, interest: Interest, user_data: u64) -> Result<()> {
    epoll_control(epoll_fd, fd, interest, user_data, libc::EPOLL_CTL_ADD)?;
    Ok(())
}

pub(crate) fn rearm(epoll_fd: i32, fd: i32, interest: Interest, user_data: u64) -> Result<()> {
    epoll_control(epoll_fd, fd, interest, user_data, libc::EPOLL_CTL_MOD)?;
    Ok(())
}

fn epoll_control(epoll_fd: i32, fd: i32, interest: Interest, user_data: u64, op: i32) -> Result<()> {
    let events = Flags::EdgeTriggered as u32 | Flags::OneShot as u32 | interest.to_u32();

    let mut event = libc::epoll_event {
        events,
        u64: user_data,
    };

    let status = unsafe { libc::epoll_ctl(epoll_fd, op, fd, &mut event as *mut libc::epoll_event) };
    let _ = res!(status);
    Ok(())
}

// -----------------------------------------------------------------------------
//     - Epoll wait -
// -----------------------------------------------------------------------------
pub fn wait(epoll_fd: i32, events: &mut [libc::epoll_event], max_events: i32, timeout: i32) -> Result<usize> {
    let result = unsafe { libc::epoll_wait(epoll_fd, events.as_mut_ptr(), max_events, timeout) };
    let result = res!(result) as usize;
    Ok(result)
}


// -----------------------------------------------------------------------------
//     - Flags -
// -----------------------------------------------------------------------------
#[repr(u32)]
pub enum Flags {
    EdgeTriggered = libc::EPOLLET as u32,
    OneShot = libc::EPOLLONESHOT as u32,
    Read = libc::EPOLLIN as u32,
    Write = libc::EPOLLOUT as u32,
    _UrgentRead = libc::EPOLLPRI as u32,
    _Error = libc::EPOLLERR as u32,
    _Hup = libc::EPOLLHUP as u32,
    RHup = libc::EPOLLRDHUP as u32,
    _Wakeup = libc::EPOLLWAKEUP as u32,
    _Exclusive = libc::EPOLLEXCLUSIVE as u32,
}

impl Flags {
    pub(crate) fn contains(val: u32, flag: Flags) -> bool {
        let flag = flag as u32;
        0 != (val & flag)
    }
}
