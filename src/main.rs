use std::io::{Read, Result};
use std::os::unix::io::AsRawFd;
use std::thread;

use netlib::{Reaction, Reactor, System, Evented};

struct MyData {
    counter: usize,
    user_fd: Evented,
}

impl MyData {
    fn new() -> Result<Self> {
        let user_fd = Evented::new()?;

        let inst = Self {
            counter: 0,
            user_fd,
        };

        Ok(inst)
    }

    fn poke(&mut self) {
        self.user_fd.poke();
    }
}

impl AsRawFd for MyData {
    fn as_raw_fd(&self) -> i32 {
        self.user_fd.as_raw_fd()
    }
}

impl Reactor for MyData {
    type Input = ();
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        eprintln!("{}", self.counter);
        self.counter += 1;
        eprintln!("reacted");
        // let mut buf = [0;128];
        // let _ = self.user_fd.read(&mut buf);
        let x = self.user_fd.rearm();
        // thread::sleep(std::time::Duration::from_secs(1));
        eprintln!("{:?}", x);
        match reaction {
            Reaction::Value(val) => Reaction::Value(val),
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(ev) => Reaction::Event(ev),
        }
    }
}

fn main() -> Result<()> {
    System::builder().finish();
    let my_data = MyData::new()?;
    System::start(my_data)?;
    Ok(())
}
