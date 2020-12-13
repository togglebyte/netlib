use std::io::ErrorKind::WouldBlock;
use std::io::{self, Error, Read, Write};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

use libc::{
    epoll_create1, epoll_ctl, epoll_event, epoll_wait, EPOLLIN, EPOLLET, EPOLLONESHOT,
    EPOLL_CTL_ADD, EPOLL_CTL_MOD,
};

pub mod connections;
pub mod errors;

fn make_misc_events(cap: usize) -> Vec<epoll_event> {
    (0..cap)
        .map(|_| epoll_event { events: 0, u64: 0 })
        .collect()
}

macro_rules! res {
    ($e:expr) => {
        match $e {
            -1 => return Err(errors::os_err()),
            val => val,
        }
    };
}

pub unsafe fn run() -> io::Result<()> {
    let listener_flags = EPOLLIN as u32 | EPOLLET as u32 | EPOLLONESHOT as u32;
    let con_flags = EPOLLIN as u32 | EPOLLET as u32 | EPOLLONESHOT as u32;

    // Epoll
    let epoll_fd = res!(epoll_create1(0));

    // Server
    let listener = TcpListener::bind("127.0.0.1:9000")?;
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();

    // Listen for read events and call accept on the listener
    let _ = res!({
        let mut event = epoll_event {
            events: listener_flags,
            u64: u64::MAX,
        };

        epoll_ctl(
            epoll_fd,
            EPOLL_CTL_ADD,
            listener_fd,
            &mut event as *mut epoll_event,
        )
    });

    // Do all the fun bits
    let mut events = make_misc_events(100);
    let mut cons = connections::Connections::with_capacity(1024);

    loop {
        // Poll events
        let event_count = res!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr(),
            events.len() as i32,
            50,
        ));

        eprintln!("count: {:?}", event_count);

        for event in game_events() {
            // handle game event
        }

        for i in 0..event_count as usize {
            let ev = &events[i];

            // -----------------------------------------------------------------------------
            //     - Accept incomming connections -
            //     Accept a connection, set it to be nonblocking
            //     and register for "read" events on the connection
            // -----------------------------------------------------------------------------
            if ev.u64 == u64::MAX {
                // Accept a new connection
                let (connection, addr) = listener.accept()?;
                connection.set_nonblocking(true)?;
                let con_fd = connection.as_raw_fd();

                // Register the con_idx as the user data
                // for the epoll event
                let con_idx = cons.insert(connection);

                let _ = res!({
                    let mut event = epoll_event {
                        events: con_flags,
                        u64: con_idx as u64,
                    };

                    epoll_ctl(
                        epoll_fd,
                        EPOLL_CTL_ADD,
                        con_fd,
                        &mut event as *mut epoll_event,
                    )
                });

                eprintln!("new connection: {:?}", addr);

                // Re-register interest
                let _ = res!({
                    let mut event = epoll_event {
                        events: listener_flags,
                        u64: u64::MAX,
                    };

                    epoll_ctl(
                        epoll_fd,
                        EPOLL_CTL_MOD,
                        listener_fd,
                        &mut event as *mut epoll_event,
                    )
                });
            } else {
                let index = ev.u64 as usize;
                let con = &mut cons[index];
                let mut buf = [0u8; 128];
                let res = con.read(&mut buf);
                match res {
                    Ok(0) => {
                        eprintln!("connection closed");
                        cons.remove(index);
                    }
                    Ok(n) => {
                        // Get message
                        let msg = std::str::from_utf8_unchecked(&buf[..n]);
                        eprintln!("{}", msg);
                        let _ = con.write(&buf[..n]);
                    }
                    Err(ref e) if e.kind() == WouldBlock => {
                        eprintln!("{:?}", "would block");
                    }
                    Err(e) => {
                        eprintln!("connection closed: {:?}", e);
                        cons.remove(index);
                    }
                }
            }
        }
    }

    // Closing epoll
    let _ = res!(libc::close(epoll_fd));
    Ok(())
}
