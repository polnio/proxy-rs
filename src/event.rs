use crate::{tcp, udp};

#[derive(Debug)]
pub enum Event {
    Tcp(tcp::Event),
    Udp(udp::Event),
}
