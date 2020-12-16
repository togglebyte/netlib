use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;

use netlib::{Evented, Reaction, Reactor, Result, System, SysEvent};

struct MyData {
    counter: usize,
    evented: Evented,
}

impl MyData {
    fn new() -> Result<Self> {
        let evented = Evented::new()?;

        let inst = Self {
            counter: 0,
            evented,
        };

        Ok(inst)
    }
}

impl AsRawFd for MyData {
    fn as_raw_fd(&self) -> i32 {
        self.evented.as_raw_fd()
    }
}

impl Reactor for MyData {
    type Input = ();
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        eprintln!("{:?}", "event happened");
        self.counter += 1;
        let x = self.evented.rearm();
        thread::sleep(std::time::Duration::from_secs(1));
        match reaction {
            Reaction::Value(val) => Reaction::Value(val),
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(ev) => Reaction::Event(ev),
        }
    }
}

fn main() -> Result<()> {
    let mut handle = System::builder().finish()?;

    let my_data = MyData::new()?;

    let mut evented = my_data.evented.clone();
    thread::spawn(move ||  {
        handle.send(SysEvent::Stop);
    });

    System::start(my_data)?;
    Ok(())
}
