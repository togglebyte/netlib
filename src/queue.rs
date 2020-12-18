use crossbeam::deque::{Steal, Stealer as CBStealer, Worker as CBWorker};

use crate::{Evented, Interest, Reaction, Reactor, Result, System};

// -----------------------------------------------------------------------------
//     - Worker -
// -----------------------------------------------------------------------------
pub struct Worker<T> {
    inner: CBWorker<T>,
    evented: Evented,
}

impl<T> Worker<T> {
    pub fn new() -> Result<Self> {
        let inner = CBWorker::new_fifo();
        let evented = Evented::new()?;
        let inst = Self { inner, evented };
        Ok(inst)
    }

    pub fn dequeue(&self) -> Stealer<T> {
        Stealer::new(self.inner.stealer(), self.evented.clone())
    }
}

impl<T> Reactor for Worker<T> {
    type Input = T;
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Value(val) => {
                self.evented.poke();
                Reaction::Value(self.inner.push(val))
            }
            Reaction::Continue => Reaction::Continue,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Stealer -
// -----------------------------------------------------------------------------
pub struct Stealer<T> {
    inner: CBStealer<T>,
    pub evented: Evented,
}

impl<T> Stealer<T> {
    fn new(inner: CBStealer<T>, evented: Evented) -> Self {
        Self { inner, evented }
    }

    pub fn arm(&mut self) -> Result<()> {
        self.evented.reactor_id = System::reserve();
        System::arm(&self.evented.fd, Interest::Read, self.evented.reactor_id)
    }
}

impl<T> Reactor for Stealer<T> {
    type Input = ();
    type Output = T;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if ev.owner != self.evented.reactor_id => Reaction::Event(ev),
            Reaction::Event(ev) => {
                let res = self.evented.rearm();
                loop {
                    let res = self.inner.steal();

                    match res {
                        Steal::Success(val) => break Reaction::Value(val),
                        Steal::Retry => continue,
                        Steal::Empty => break Reaction::Continue,
                    }
                }
            }
            Reaction::Value(()) | Reaction::Continue => Reaction::Continue,
        }
    }
}
