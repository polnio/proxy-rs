use std::io;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum Event {
    Connection {
        client_addr: SocketAddr,
        remote_addr: SocketAddr,
    },
    ConnectionError {
        error: io::Error,
    },
    Disconnection {
        client_addr: SocketAddr,
        remote_addr: SocketAddr,
    },
    Message {
        from_addr: SocketAddr,
        to_addr: SocketAddr,
        message: String,
    },
    MessageError {
        from_addr: SocketAddr,
        to_addr: SocketAddr,
        error: io::Error,
    },
}
