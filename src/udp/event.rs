use std::io;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum Event {
    Message(Message),
    MessageError(MessageError),
}

#[derive(Debug)]
pub struct Message {
    pub from_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub to_addr: SocketAddr,
    pub message: String,
}
impl From<Message> for Event {
    fn from(event: Message) -> Self {
        Event::Message(event)
    }
}

#[derive(Debug)]
pub struct MessageError {
    pub from_addr: Option<SocketAddr>,
    pub local_addr: SocketAddr,
    pub to_addr: SocketAddr,
    pub error: io::Error,
}
impl From<MessageError> for Event {
    fn from(event: MessageError) -> Self {
        Event::MessageError(event)
    }
}
