use crossbeam::channel::{bounded, Receiver as CBReceiver, Sender as CBSender};

use crate::{Evented, Interest, Reaction, Reactor, Result, System};

// -----------------------------------------------------------------------------
//     - Broadcaster -
// -----------------------------------------------------------------------------
pub struct Broadcaster<T: Clone> {
    subscribers: Vec<(Evented, CBSender<T>)>,
    event_cap: usize,
}

impl<T: Clone> Broadcaster<T> {
    pub fn new(event_cap: usize) -> Self {
        let inst = Self {
            subscribers: Vec::new(),
            event_cap,
        };

        inst
    }
    pub fn send(&mut self, val: T) {
        self.subscribers.iter_mut().for_each(|(e, tx)| {
            e.poke();
            tx.send(val.clone());
        });
    }

    pub fn receiver(&mut self) -> Result<Receiver<T>> {
        let evented = Evented::new()?;
        let (tx, rx) = bounded(self.event_cap);
        self.subscribers.push((evented.clone(), tx));
        let rec = Receiver::new(rx, evented);
        Ok(rec)
    }
}

impl<T: Clone> Reactor for Broadcaster<T> {
    type Input = T;
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Value(val) => {
                self.send(val);
                Reaction::Value(())
            }
            Reaction::Continue => Reaction::Continue,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Reveiver -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Receiver<T> {
    evented: Evented,
    rx: CBReceiver<T>,
}

impl<T> Receiver<T> {
    fn new(rx: CBReceiver<T>, evented: Evented) -> Self {
        Self { rx, evented }
    }

    fn rcv(&self) {
        self.rx.try_recv();
    }

    pub fn reactor_id(&self) -> u64 {
        self.evented.reactor_id
    }

    pub fn arm(&mut self) -> Result<()> {
        self.evented.reactor_id = System::reserve();
        System::arm(&self.evented.fd, Interest::Read, self.evented.reactor_id)
    }
}

impl<T> Reactor for Receiver<T> {
    type Input = ();
    type Output = Result<T>;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if ev.owner != self.evented.reactor_id => Reaction::Event(ev),
            Reaction::Value(()) | Reaction::Continue => Reaction::Continue,
            Reaction::Event(ev) => {
                self.evented.consume_event();

                let val = match self.rx.recv() {
                    Ok(val) => Ok(val),
                    Err(e) => Err(e.into()),
                };

                Reaction::Value(val)
            }
        }
    }
}
