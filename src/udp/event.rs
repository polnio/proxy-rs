use std::net::SocketAddr;

#[derive(Debug)]
pub enum Event {
    Message {
        from_addr: SocketAddr,
        local_addr: SocketAddr,
        to_addr: SocketAddr,
        message: String,
    },
    MessageError {
        from_addr: Option<SocketAddr>,
        local_addr: SocketAddr,
        to_addr: SocketAddr,
        error: std::io::Error,
    },
}
