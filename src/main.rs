use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

use netlib::queue::{Stealer, Worker};
use netlib::{Evented, Reaction, Reactor, Result, SysEvent, System, Timer};

// -----------------------------------------------------------------------------
//     - Provider -
//     Aka the worker
// -----------------------------------------------------------------------------
struct Provider {
    worker: Worker<usize>,
    timer: Timer,
}

impl Provider {
    fn new() -> Result<Self> {
        let inst = Self {
            worker: Worker::new()?,
            timer: Timer::new(Duration::new(2, 0), Some(Duration::new(1, 0)))?,
        };

        Ok(inst)
    }

    fn send(&mut self, val: usize) {
        self.worker.react(Reaction::Value(val));
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
                self.send(123);
                Reaction::Continue
            }
            Reaction::Value(val) => Reaction::Value(val),
            Reaction::Continue => Reaction::Continue,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Maion -
// -----------------------------------------------------------------------------
fn main() -> Result<()> {
    System::builder().finish();

    let provider = Provider::new()?;

    let thread_count = 10;

    for thread_id in 0..thread_count {
        let mut stealer = provider.worker.dequeue();
        thread::spawn(move || {
            System::builder().finish();
            let r = stealer.map(|val| eprintln!("{} | {}", thread_id, val));
            System::start(r);
        });
    }

    System::start(provider)?;
    Ok(())
}
