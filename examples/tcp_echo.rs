// use std::os::unix::io::AsRawFd;
// use std::io::{Write, Read, Result};
use crate::Result;

// use netlib::{Interest, Reaction, Reactor, System};
// use netlib::net::tcp::{TcpListener, TcpStream};

// const RESPONSE: &'static [u8] = b"hello";

// struct Connections {
//     inner: Vec<(u64, TcpStream, [u8;256], usize)>,
// }

// // fn danger_buffer() -> [u8;256] {
// // }

// impl Reactor for Connections {
//     type Input = TcpStream;
//     type Output = ();

//     fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
//         match reaction {
//             Reaction::Value(stream) => {
//                 self.inner.push((stream.id, stream, [0u8;256], 0));
//             }
//             Reaction::Event(event) => {
//                 eprintln!("{:?}", event);
//                 self.inner.iter_mut().filter(|(id, ..)| *id == event.owner).map(|(_, stream, buf, bytes_read)| {
//                     if event.read {
//                         match stream.read(buf) {
//                             Ok(0) => eprintln!("{:?}", "connection closed"),
//                             Ok(n) => {
//                                 let mut junk_buf = [0u8;1024];
//                                 *bytes_read = n;
//                                 stream.interest = Interest::ReadWrite;
//                                 stream.rearm();
//                             }
//                             Err(e) => { }
//                         }
//                     }

//                     if event.write {
//                         if *bytes_read > 0 {
//                             let _ = stream.write(&RESPONSE);
//                         }
//                         stream.interest = Interest::Read;
//                         stream.close();
//                         // stream.rearm();
//                     }
//                 }).count();
//             }
//             _ => {}
//         }

//         Reaction::Value(())
//     }
// }

fn main() -> Result<()> {
//     // Listener
//     System::builder().finish();

//     let listener = TcpListener::bind("127.0.0.1:9000")?
//         .map(|(stream, _)| stream)
//         .map(|s| {
//             s.set_nonblocking(true);
//             TcpStream::new(s, Interest::ReadWrite).unwrap()
//         });

//     // Connections
//     let connections = Connections { inner: Vec::with_capacity(2) };

//     let everything = listener.chain(connections);

//     match System::start(everything) {
//         Ok(()) => eprintln!("{:?}", "started"),
//         Err(e) => eprintln!("err: {:?}", e),
//     }

    Ok(())
}
