use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::io::{Read, Result, Write};
use std::thread;

use netlib::net::tcp::{TcpListener, TcpStream};
use netlib::{Interest, Reaction, Reactor, System};

// Connection: Closed
// const RESPONSE: &'static [u8] = br#"HTTP/1.1 200 OK
// Server: Lark
// Content-Length: 600
// content-type: text/html; charset=UTF-8

// hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello     
// "#;
static RESPONSE: &'static [u8] = b"HTTP/1.1 200 OK\nContent-Length: 13\n\nhello world\n\n";    

pub struct HttpServer {
    thread_id: usize,
    msg_count: usize,
    b: [u8; 1024],
    con: HashMap<u64, TcpStream>,
}

impl HttpServer {
    pub fn new(thread_id: usize) -> Self {
        Self {
            thread_id,
            msg_count: 0,
            b: [0; 1024],
            con: HashMap::new(),
        }
    }
}

impl Reactor for HttpServer {
    type Input = TcpStream;
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Value(stream) => {
                self.con.insert(stream.id, stream);
                Reaction::Continue
            }
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(ev) => {
                // if !ev.write {
                //     return Reaction::Continue;
                // }

                let b = &mut self.b;
                let mut close = false;
                if ev.read {
                    self.con.get_mut(&ev.owner).map(|con| {
                        con.readable = ev.read;
                        while con.readable {
                            con.read(b);
                        }
                    });
                }

                if let Some(mut con) = self.con.remove(&ev.owner) {
                    let mut cnt = 0;
                    con.writable = ev.write;
                    con.readable = ev.read;
                    while con.writable {
                        con.write(&RESPONSE);
                    }
                }
                Reaction::Continue
            }
        }
    }
}

fn main() -> Result<()> {
    let thread_count = 8;
    let mut handles = Vec::new();
    for thread_id in 0..thread_count {
        let h = thread::spawn(move || -> Result<()> {
            // Initialise the system
            System::builder().finish();

            let listener = TcpListener::bind("127.0.0.1:9000")?
                .map(Result::unwrap)
                .map(|(stream, _)| {
                stream.set_nonblocking(true);
                TcpStream::new(stream, Interest::ReadWrite).unwrap()
            });

            let server = listener.chain(HttpServer::new(thread_id));

            // Start the server
            System::start(server);

            Ok(())
        });

        handles.push(h);
    }

    handles.into_iter().map(|h| h.join()).count();

    Ok(())
}
