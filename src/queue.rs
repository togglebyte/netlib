use crossbeam::deque::{Steal, Stealer as CBStealer, Worker as CBWorker};

use crate::{Evented, Interest, Reaction, Reactor, Result, System};

// TODO:
// 1. Track all stealers as FDs in a vec
// 2. Poke each FD on change
// -----------------------------------------------------------------------------
//     - Worker -
// -----------------------------------------------------------------------------
pub struct Worker<T> {
    inner: CBWorker<T>,
    stealers: Vec<Evented>,
    // evented: Evented,
    current_stealer_id: usize,
}

impl<T> Worker<T> {
    pub fn new() -> Result<Self> {
        let inner = CBWorker::new_fifo();

        // let evented = Evented::new()?;
        let inst = Self {
            inner,
            stealers: Vec::new(),
            current_stealer_id: 0,
        };

        Ok(inst)
    }

    pub fn dequeue(&mut self) -> Result<Stealer<T>> {
        self.current_stealer_id += 1;
        let evented = Evented::new()?;
        self.stealers.push(evented.clone());
        let inst = Stealer::new(
            self.inner.stealer(),
            evented,
            self.current_stealer_id,
        );

        Ok(inst)
    }

    pub fn send(&mut self, val: T) {
        self.stealers.iter_mut().for_each(|s| { s.poke(); });
        self.inner.push(val)
    }
}

impl<T> Reactor for Worker<T> {
    type Input = T;
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Value(val) => Reaction::Value(self.send(val)),
            Reaction::Continue => Reaction::Continue,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Stealer -
// -----------------------------------------------------------------------------
pub struct Stealer<T> {
    inner: CBStealer<T>,
    id: usize,
    pub evented: Evented,
}

impl<T> Stealer<T> {
    fn new(inner: CBStealer<T>, evented: Evented, id: usize) -> Self {
        Self { inner, evented, id }
    }

    pub fn arm(&mut self) -> Result<()> {
        self.evented.reactor_id = System::reserve();
        System::arm(&self.evented.fd, Interest::Read, self.evented.reactor_id)
    }
}

impl<T> Reactor for Stealer<T> {
    type Input = ();
    type Output = Result<T>;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if ev.owner != self.evented.reactor_id => Reaction::Event(ev),
            Reaction::Event(ev) => {
                if let Err(e) = self.evented.consume_event() {
                    return Reaction::Value(Err(e));
                }

                loop {
                    match self.inner.steal() {
                        Steal::Success(val) => break Reaction::Value(Ok(val)),
                        Steal::Retry => continue,
                        Steal::Empty => break Reaction::Continue,
                    }
                }
            }
            Reaction::Value(()) | Reaction::Continue => Reaction::Continue,
        }
    }
}
