use std::cell::RefCell;
use std::io::Result;
use std::os::unix::io::AsRawFd;

use crate::{Reaction, Reactor};

mod identities;
mod epoll;

use identities::Identities;
use epoll::Flags;
pub use epoll::Interest;

// -----------------------------------------------------------------------------
//     - TLS System -
// -----------------------------------------------------------------------------
thread_local! {
    pub static SYSTEM: RefCell<System> = RefCell::new(System::empty());
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
    pub fn finish(self) {
        let reactor_ids = self.id_capacity.unwrap_or(1024);
        let event_cap = self.event_cap.unwrap_or(1024);

        let sys = System::init(event_cap, reactor_ids);

        SYSTEM.with(|existing| *existing.borrow_mut() = sys);
    }
}

/// A system is core to run the reactors.
/// The system is responsible for polling epoll events, and propagate these
/// events to the reactors.
pub struct System {
    epoll_fd: i32,
    identities: Identities,
    event_cap: usize,
}

impl System {
    fn empty() -> Self {
        Self {
            epoll_fd: 0,
            identities: Identities::empty(),
            event_cap: 0,
        }
    }

    /// This has to happen before a system is used.
    fn init(event_cap: usize, id_cap: usize) -> Self {
        let epoll_fd = epoll::create().expect("Failed to get epoll file descriptor");
        Self { 
            epoll_fd,
            event_cap,
            identities: Identities::with_capacity(id_cap),
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
        SYSTEM.with(|sys| sys.borrow_mut().identities.reserve())
    }

    /// Free an id for a reactor. 
    /// This should happen when the reactor is no longer in use.
    pub(crate) fn free(id: u64) {
        SYSTEM.with(|sys| sys.borrow_mut().identities.free(id))
    }

    /// Register an intereset for a reactor with epol.
    pub fn arm(as_fd: &impl AsRawFd, interest: epoll::Interest, reactor_id: u64) -> Result<()> {
        SYSTEM.with(|sys| {
            epoll::arm(
                sys.borrow().epoll_fd,
                as_fd.as_raw_fd(),
                interest,
                reactor_id,
            )
        })
    }

    /// Rearm the reactor with epoll.
    /// Since this is operating in one-shot mode this should happen after an event 
    /// is passed to a reactor.
    pub fn rearm(as_fd: &impl AsRawFd, interest: epoll::Interest, reactor_id: u64) -> Result<()> {
        SYSTEM.with(|sys| {
            epoll::rearm(
                sys.borrow().epoll_fd,
                as_fd.as_raw_fd(),
                interest,
                reactor_id,
            )
        })
    }

    /// Start polling for events.
    pub fn start<T>(mut reactor: T) -> Result<()>
    where
        T: Reactor<Input = ()>,
    {
        let capacity = SYSTEM.with(|sys| sys.borrow().event_cap);
        let mut events: Vec<libc::epoll_event> = Vec::with_capacity(capacity);
        unsafe { events.set_len(capacity) };

        loop {
            let count = SYSTEM
                .with(|sys| epoll::wait(sys.borrow().epoll_fd, &mut events, capacity as i32))?;

            for epoll_event in events.drain(..count) {
                let event = crate::Event {
                    read: Flags::contains(epoll_event.events, Flags::Read),
                    write: Flags::contains(epoll_event.events, Flags::Write),
                    owner: epoll_event.u64,
                };

                reactor.react(Reaction::Event(event));
            }

            unsafe { events.set_len(capacity) };
        }
    }

    /// Shut down the system and close the epoll file descriptor.
    pub fn shutdown() -> Result<()> {
        SYSTEM.with(|sys| epoll::close(sys.borrow().epoll_fd))?;
        Ok(())
    }
}
