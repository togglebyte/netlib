use std::marker::PhantomData;

use crossbeam::channel::{bounded, Receiver as CBReceiver, Sender as CBSender};

use crate::{Evented, Interest, Reaction, Reactor, Result, System};
use crate::system::{Armed, Unarmed};

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

    pub fn receiver(&mut self) -> Result<Receiver<Unarmed, T>> {
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
//     - Receiver -
// -----------------------------------------------------------------------------
pub type ArmedReceiver<T> = Receiver<Armed, T>;

#[derive(Debug, Clone)]
pub struct Receiver<U, T> {
    evented: Evented,
    rx: CBReceiver<T>,
    _p: PhantomData<U>,
}

impl<T> Receiver<Armed, T> {
    pub fn rcv(&self) -> Result<T> {
        let val = self.rx.try_recv()?;
        Ok(val)
    }

    pub fn reactor_id(&self) -> u64 {
        self.evented.reactor_id
    }
}

impl<T> Receiver<Unarmed, T> {
    fn new(rx: CBReceiver<T>, evented: Evented) -> Self {
        Self { rx, evented, _p: PhantomData }
    }

    pub fn arm(mut self) -> Result<Receiver<Armed, T>> {
        self.evented.reactor_id = System::reserve();
        System::arm(&self.evented.fd, Interest::Read, self.evented.reactor_id)?;

        let inst = Receiver {
            evented: self.evented,
            rx: self.rx,
            _p: PhantomData,
        };

        Ok(inst)
    }
}

impl<T> Reactor for Receiver<Armed, T> {
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
