use std::net::SocketAddr;

#[derive(Debug)]
pub enum Event {
    Message {
        from_addr: SocketAddr,
        to_addr: SocketAddr,
        message: String,
    },
    SendError {
        local_addr: SocketAddr,
        to_addr: SocketAddr,
        error: std::io::Error,
    },
    RecvError {
        local_addr: SocketAddr,
        error: std::io::Error,
    },
}
