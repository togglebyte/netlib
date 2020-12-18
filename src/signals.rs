use crossbeam::channel::{unbounded, Receiver as CBReceiver, Sender as CBSender};

use crate::{Evented, Reaction, Reactor, Result};

pub fn signal<T: Clone>() -> Result<(Sender<T>, Receiver<T>)> {
    let (tx, rx) = unbounded();

    let tx = Sender {
        tx,
        evented: Evented::new()?,
    };

    let rx = Receiver {
        rx,
        evented: tx.evented.clone(),
    };

    Ok((tx, rx))
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
    fn rcv(&self) {
        self.rx.try_recv();
    }

    pub fn reactor_id(&self) -> u64 {
        self.evented.reactor_id
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

// -----------------------------------------------------------------------------
//     - Sender -
// -----------------------------------------------------------------------------
pub struct Sender<T: Clone> {
    evented: Evented,
    tx: CBSender<T>,
}

impl<T: Clone> Sender<T> {
    pub fn send(&mut self, val: T) {
        self.tx.send(val.clone());
        self.evented.poke();
    }
}
