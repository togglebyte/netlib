use std::cell::RefCell;
use std::os::unix::io::AsRawFd;

use crate::{Reaction, Reactor, Result};
use crate::signals::{Sender, Receiver, signal};

mod identities;
mod epoll;
pub(crate) mod evented;

use identities::Identities;
use epoll::Flags;
pub use epoll::Interest;

// -----------------------------------------------------------------------------
//     - TLS System -
// -----------------------------------------------------------------------------
thread_local! {
    pub static SYSTEM: RefCell<SystemState> = RefCell::new(SystemState::Empty);
}

#[derive(Debug, Clone, Copy)] 
pub enum SysEvent {
    Stop,
}

pub enum SystemState {
    Empty,
    Running(System),
    Stopped(System),
}

// -----------------------------------------------------------------------------
//     - System builder -
// -----------------------------------------------------------------------------
/// `SystemBuilder` shouldn't be used directly, 
/// but rather through `System::builder()`.
pub struct SystemBuilder {
    event_cap: Option<usize>,
    id_capacity: Option<usize>,
}

impl SystemBuilder {
    /// Set the event capacity.
    /// This is the number of events that will be passed to epoll.
    pub fn with_capacity(&mut self, cap: usize) -> &mut Self {
        self.event_cap = Some(cap);
        self
    }

    /// Pre allocate the number of reactor identifiers.
    /// This can grow as it is inherently a `Vec<()>`.
    pub fn reactor_cap(&mut self, cap: usize) -> &mut Self {
        self.id_capacity = Some(cap);
        self
    }

    /// Finish the `System` and set it up for the local thread.
    pub fn finish(self) {//-> Result<Sender<SysEvent>> {
        let reactor_ids = self.id_capacity.unwrap_or(1024);
        let event_cap = self.event_cap.unwrap_or(10);

        let sys = System::init(event_cap, reactor_ids);

        SYSTEM.with(|existing| *existing.borrow_mut() = SystemState::Running(sys));

        // let (tx, rx) = signal()?;
        // SYSTEM.with(|existing| {
        //     existing.borrow_mut().rx = Some(rx);
        //     eprintln!("{:?}", existing.borrow_mut().initialized );
        // });

        // Ok(tx)
    }
}

/// A system is core to run the reactors.
/// The system is responsible for polling epoll events, and propagate these
/// events to the reactors.
pub struct System {
    epoll_fd: i32,
    identities: Identities,
    event_cap: usize,
    rx: Option<Receiver<SysEvent>>,
}

impl System {
    /// This has to happen before a system is used.
    fn init(event_cap: usize, id_cap: usize) -> Self {
        let epoll_fd = epoll::create().expect("Failed to get epoll file descriptor");
        Self { 
            epoll_fd,
            event_cap,
            identities: Identities::with_capacity(id_cap),
            rx: None,
        }
    }

    /// Start creating a system:
    /// ```
    /// # use netlib::System;
    /// // Init system before use
    /// System::builder().finish();
    /// // System is now available for use.
    /// ```
    pub fn builder() -> SystemBuilder {
        SystemBuilder {
            event_cap: None,
            id_capacity: None,
        }
    }

    /// Reserve an id for a reactor
    pub(crate) fn reserve() -> u64 {
        SYSTEM.with(|sys| match *sys.borrow_mut() {
            SystemState::Empty => panic!("System is uninitialized"),
            SystemState::Running(ref mut s) => s.identities.reserve(),
            SystemState::Stopped(_) => panic!("System stopped"),
        })
    }

    /// Free an id for a reactor. 
    /// This should happen when the reactor is no longer in use.
    pub(crate) fn free(id: u64) {
        SYSTEM.with(|sys| match *sys.borrow_mut() {
            SystemState::Empty => panic!("System is uninitialized"),
            SystemState::Running(ref mut s) => s.identities.free(id),
            SystemState::Stopped(_) => panic!("System stopped"),
        });
    }

    /// Register an intereset for a reactor with epol.
    pub fn arm(as_fd: &impl AsRawFd, interest: epoll::Interest, reactor_id: u64) -> Result<()> {
        SYSTEM.with(|sys| match *sys.borrow() {
            SystemState::Empty => panic!("System is uninitialized"),
            SystemState::Running(ref sys) => {
                epoll::arm(
                    sys.epoll_fd,
                    as_fd.as_raw_fd(),
                    interest,
                    reactor_id,
                )
            }
            SystemState::Stopped(_) => panic!("System stopped"),
        });
        Ok(())
    }

    /// Rearm the reactor with epoll.
    /// Since this is operating in one-shot mode this should happen after an event 
    /// is passed to a reactor.
    pub fn rearm(as_fd: &impl AsRawFd, interest: epoll::Interest, reactor_id: u64) -> Result<()> {
        SYSTEM.with(|sys| match *sys.borrow() {
            SystemState::Empty => panic!("System is uninitialized"),
            SystemState::Running(ref sys) => {
                epoll::rearm(
                    sys.epoll_fd,
                    as_fd.as_raw_fd(),
                    interest,
                    reactor_id,
                )
            }
            SystemState::Stopped(_) => panic!("System stopped"),
        });
        Ok(())
    }

    /// Start polling for events.
    pub fn start<T>(mut reactor: T) -> Result<()>
    where
        T: Reactor<Input = ()>,
    {
        let event_cap = SYSTEM.with(|sys| match *sys.borrow() {
            SystemState::Empty => panic!("System is uninitialized"),
            SystemState::Stopped(_) => panic!("System stopped"),
            SystemState::Running(ref s) => s.event_cap,
        });

        // let mut rx = SYSTEM.with(|sys| sys.borrow_mut().rx.take());//.expect("this should never be None");
        let mut events: Vec<libc::epoll_event> = Vec::with_capacity(event_cap);
        unsafe { events.set_len(event_cap) };

        let timeout = 0;
        // Have zero ms timeout for epoll.
        // Have timeout at application level.
        //
        // 1. Check epoll events
        // 2. Check user defined events
        // 3. ??? <-- don't cook the fish
        'system: loop {
            let count = SYSTEM.with(|sys| match *sys.borrow() {
                SystemState::Empty => panic!("System is uninitialized"),
                SystemState::Stopped(_) => panic!("System stopped"),
                SystemState::Running(ref sys) => epoll::wait(sys.epoll_fd, &mut events, event_cap as i32, timeout)
            })?;

            for epoll_event in events.drain(..count) {
                let event = crate::Event {
                    read: Flags::contains(epoll_event.events, Flags::Read),
                    write: Flags::contains(epoll_event.events, Flags::Write),
                    owner: epoll_event.u64,
                };

                eprintln!("{:?}", epoll_event.events);

                let reaction = Reaction::Event(event);

                // Shut down the system by breaking the loop.
                // if epoll_event.u64 == rx.reactor_id() {
                //     if Flags::contains(epoll_event.events, Flags::Read) {
                //         if let Reaction::Value(sys_ev) = rx.react(reaction) {
                //             match sys_ev {
                //                 Ok(SysEvent::Stop) => break 'system,
                //                 _ => {}
                //             }
                //         }
                //     }
                // } else {
                    reactor.react(reaction);
                // }
            }

            unsafe { events.set_len(event_cap) };
            // Run game loop
        }

        System::shutdown()
    }

    /// Shut down the system and close the epoll file descriptor.
    pub fn shutdown() -> Result<()> {
        // TODO: implement this
        // SYSTEM.with(|sys| epoll::close(sys.borrow().epoll_fd))?;
        Ok(())
    }
}
