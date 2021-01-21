use std::marker::PhantomData;

use crossbeam::channel::{bounded, Receiver as CBReceiver, Sender as CBSender};

use crate::{Evented, Interest, Reaction, Reactor, Result, System};
use crate::system::{Armed, Unarmed};
use crate::mpsc::{Sender, Receiver};

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
        unimplemented!(); // TODO: Need to solve armed / unarmed first
        // Ok(rec)
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

