use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;

use netlib::{Evented, Reaction, Reactor, Result, System, SysEvent};

struct MyData {
    evented: Evented,
}

impl MyData {
    fn new() -> Result<Self> {
        let evented = Evented::new()?;

        let inst = Self {
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
        match reaction {
            Reaction::Value(val) => Reaction::Value(val),
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(ev) if ev.owner != self.evented.reactor_id => Reaction::Event(ev),
            Reaction::Event(ev) => {
                eprintln!("reaction happened");
                self.evented.do_read();
                let res = self.evented.rearm();
                Reaction::Event(ev) 
            }
        }
    }
}

fn main() -> Result<()> {
    System::builder().finish();

    let my_data = MyData::new()?;

    let mut evented = my_data.evented.clone();
    thread::spawn(move ||  {
        loop {
            thread::sleep_ms(1000);
            evented.poke();
        }
    });

    System::start(my_data)?;
    Ok(())
}
