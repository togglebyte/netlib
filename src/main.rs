use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;

use netlib::queue::{Stealer, Worker};
use netlib::{Evented, Reaction, Reactor, Result, SysEvent, System};

struct Provider {
    worker: Worker<usize>,
}

impl Provider {
    fn send(&mut self, val: usize) {
        self.worker.react(Reaction::Value(val));
    }
}

fn main() -> Result<()> {
    System::builder().finish();

    let provider = Provider {
        worker: Worker::new()?,
    };

    let thread_count = 10;

    for thread_id in 0..thread_count {
        let mut stealer = provider.worker.dequeue();
        thread::spawn(move || {
            System::builder().finish();
            let r = stealer.map(|val| eprintln!("{} | {}", thread_id, val));
            System::start(r);
        });
    }

    // System::start(r)?;
    Ok(())
}
