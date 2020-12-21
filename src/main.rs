use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

use netlib::queue::{Stealer, Worker};
use netlib::signals::{signal, Sender};
use netlib::{Evented, Reaction, Reactor, Result, SysEvent, System, Timer};

// -----------------------------------------------------------------------------
//     - Provider -
//     Aka the worker
// -----------------------------------------------------------------------------
struct Provider {
    // producer: Worker<usize>,
    producer: Sender<usize>,
    count: usize,
    timer: Timer,
}

impl Provider {
    fn new(producer: Sender<usize>) -> Result<Self> {
        let inst = Self {
            producer,
            // producer: Worker::new()?,
            timer: Timer::new(Duration::new(2, 0), Some(Duration::from_millis(1000)))?,
            count: 0,
        };

        Ok(inst)
    }

    fn send(&mut self, val: usize) {
        self.producer.react(Reaction::Value(val));
    }
}

impl Reactor for Provider {
    type Input = ();
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if ev.owner != self.timer.reactor_id => Reaction::Event(ev),
            Reaction::Event(ev) => {
                self.timer.consume_event();
                self.send(self.count);
                self.count += 1;
                Reaction::Continue
            }
            Reaction::Value(val) => Reaction::Value(val),
            Reaction::Continue => Reaction::Continue,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Main -
// -----------------------------------------------------------------------------
fn main() -> Result<()> {
    System::builder().finish();

    let (tx, rx) = signal()?;
    let provider = Provider::new(tx)?;

    let thread_count = 4;

    for thread_id in 0..thread_count {
        // let mut receiver = provider.producer.dequeue();
        let mut receiver = rx.clone();
        thread::spawn(move || {
            System::builder().finish();
            receiver.arm();
            let r = receiver.map(|val| eprintln!("{} | {}", thread_id, val.unwrap()));
            System::start(r);
        });
    }

    System::start(provider)?;
    Ok(())
}
