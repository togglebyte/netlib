use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

use netlib::queue::{Stealer, Worker};
use netlib::broadcast::Broadcaster;
use netlib::{Evented, Reaction, Reactor, Result, SysEvent, System, Timer};

use rand::prelude::*;

const THREAD_COUNT: usize = 3;

// -----------------------------------------------------------------------------
//     - Provider -
//     Aka the worker
// -----------------------------------------------------------------------------
struct Provider {
    // producer: Worker<usize>,
    producer: Broadcaster<usize>,
    count: usize,
    timer: Timer,
}

impl Provider {
    fn new(producer: Broadcaster<usize>) -> Result<Self> {
        let inst = Self {
            producer,
            // producer: Worker::new()?,
            timer: Timer::new(Duration::new(1, 0), Some(Duration::from_millis(100)))?,
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

    let tx = Broadcaster::new(100);
    let mut provider = Provider::new(tx)?;

    for thread_id in 0..THREAD_COUNT {
        // let mut receiver = provider.producer.dequeue();
        let mut receiver = provider.producer.receiver()?;
        thread::spawn(move || {
            System::builder().finish();
            receiver.arm();
            let r = receiver.map(|val| {
                let s = thread_rng().gen_range(100..3000);
                thread::sleep(Duration::from_millis(s));
                eprintln!("{} | {} | sleep: {}", thread_id, val.unwrap(), s);
            });
            System::start(r);
        });
    }

    System::start(provider)?;
    Ok(())
}
