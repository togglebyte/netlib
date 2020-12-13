# Netlib

## Requirements

* Should work with `std`:
  * TcpListener / TcpStream
  * Unix domain socket
  * Stdin / Stdout
* Should be batteries included
    * Buffers
    * Connection<T: Read + Write, U: Buffer>
* As few dependencies as possible
* Should not allow blocking sockets

## Api

Questions: error handling?

```rust
// Create a game loop (ticks every N ms)
let game_loop = GameLoopTimer::new();

// Setup networking
let listener = TcpListener::bind("0.0.0.0:9000")?;
let connection_handler = ConnectionHandler::new();

// listener -> connection handler
let networking = listener.chain(connection_handler);

// join the game loop with the networking component
let game = game_loop.join(networking)

system.start(game);
```

System -> [listener] -> [tcp stream] -> ?how do we register this stream

* Make System TLS
