use std::io;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum Event {
    Connection(Connection),
    ConnectionError(ConnectionError),
    Disconnection(Disconnection),
    Message(Message),
    MessageError(MessageError),
}

#[derive(Debug)]
pub struct Connection {
    pub client_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
}
impl From<Connection> for Event {
    fn from(value: Connection) -> Self {
        Event::Connection(value)
    }
}

#[derive(Debug)]
pub struct ConnectionError {
    pub local_addr: SocketAddr,
    pub error: io::Error,
}
impl From<ConnectionError> for Event {
    fn from(value: ConnectionError) -> Self {
        Event::ConnectionError(value)
    }
}

#[derive(Debug)]
pub struct Disconnection {
    pub client_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
}
impl From<Disconnection> for Event {
    fn from(value: Disconnection) -> Self {
        Event::Disconnection(value)
    }
}

#[derive(Debug)]
pub struct Message {
    pub from_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub to_addr: SocketAddr,
    pub message: String,
}
impl From<Message> for Event {
    fn from(value: Message) -> Self {
        Event::Message(value)
    }
}

#[derive(Debug)]
pub struct MessageError {
    pub from_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub to_addr: SocketAddr,
    pub error: io::Error,
}
impl From<MessageError> for Event {
    fn from(value: MessageError) -> Self {
        Event::MessageError(value)
    }
}
