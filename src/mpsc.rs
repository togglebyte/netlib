use std::marker::PhantomData;

use crossbeam::channel::{bounded, Receiver as CBReceiver, Sender as CBSender};

use crate::system::{Armed, Interest, System, Unarmed};
use crate::{Evented, Reaction, Reactor, Result};

pub fn channel<T>(cap: usize) -> (Sender<T>, ArmedReceiver<T>) {
    let fd = Evented::new().unwrap();
    let (tx, rx) = bounded(cap);
    (Sender::new(tx, fd), Receiver::new(rx, fd))
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

    pub fn new(rx: CBReceiver<T>, evented: Evented) -> Self {
        Self {
            rx,
            evented,
            _p: PhantomData,
        }
    }

    // pub fn arm(mut self) -> Result<Receiver<Armed, T>> {
    //     self.evented.reactor_id = System::reserve();
    //     System::arm(&self.evented.fd, Interest::Read, self.evented.reactor_id)?;

    //     let inst = Receiver {
    //         evented: self.evented,
    //         rx: self.rx,
    //         _p: PhantomData,
    //     };

    //     Ok(inst)
    // }
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

// -----------------------------------------------------------------------------
//     - Sender -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Sender<T> {
    sender: CBSender<T>,
    evented: Evented,
}

impl<T> Sender<T> {
    pub fn send(&mut self, val: T) -> Result<()> {
        self.evented.poke();
        self.sender.try_send(val);
        Ok(())
    }

    pub fn new(sender: CBSender<T>, evented: Evented) -> Self {
        Self {
            sender,
            evented,
        }
    }
}
 
impl<T> Reactor for Sender<T> {
    type Input = T;
    type Output = Result<()>;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Value(val) => Reaction::Value(self.send(val)),
            Reaction::Continue => Reaction::Continue,
        }
    }
}
