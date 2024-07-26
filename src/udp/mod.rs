mod builder;
mod event;

use std::collections::VecDeque;
use std::net::{IpAddr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

pub use self::builder::ProxyBuilder;
pub use self::event::Event;

fn are_addrs_eq(addr1: &SocketAddr, addr2: &SocketAddr) -> bool {
    let ip1 = match addr1.ip() {
        IpAddr::V4(ip) => ip.to_ipv6_mapped(),
        IpAddr::V6(ip) => ip,
    };
    let ip2 = match addr2.ip() {
        IpAddr::V4(ip) => ip.to_ipv6_mapped(),
        IpAddr::V6(ip) => ip,
    };
    ip1 == ip2 && addr1.port() == addr2.port()
}

#[derive(Debug)]
pub struct Proxy {
    buffer_size: usize,
    remote_addrs: Vec<SocketAddr>,

    socket: UdpSocket,
    event_sender: Option<mpsc::Sender<Event>>,

    queue: VecDeque<(String, SocketAddr)>,
}

impl Proxy {
    pub fn builder() -> ProxyBuilder {
        ProxyBuilder::new()
    }

    pub async fn run(mut self) {
        while let Some((msg, client_addr)) = self.get_message().await {
            self.send_message_to_remote(&msg, &client_addr).await;
            self.send_event(Event::from(event::Message {
                from_addr: client_addr,
                local_addr: self.local_addr(),
                to_addr: self.remote_addr(),
                message: msg,
            }))
            .await;

            let Some(reply) = self.recv_reply().await else {
                continue;
            };
            self.send_message_to_client(&reply, &client_addr).await;
            self.send_event(Event::from(event::Message {
                from_addr: self.remote_addr(),
                local_addr: self.local_addr(),
                to_addr: client_addr,
                message: reply,
            }))
            .await;
        }
    }

    async fn get_message(&mut self) -> Option<(String, SocketAddr)> {
        if let Some(result) = self.queue.pop_front() {
            Some(result)
        } else {
            self.recv_message().await
        }
    }

    async fn recv_message(&self) -> Option<(String, SocketAddr)> {
        let mut buf = vec![0; self.buffer_size];
        let (len, addr) = match self.socket.recv_from(&mut buf).await {
            Ok(result) => result,
            Err(error) => {
                self.send_event(Event::from(event::MessageError {
                    from_addr: None,
                    local_addr: self.local_addr(),
                    to_addr: self.remote_addr(),
                    error,
                }))
                .await;
                return None;
            }
        };
        let msg = String::from_utf8_lossy(&buf[..len]).to_string();
        Some((msg, addr))
    }

    async fn recv_reply(&mut self) -> Option<String> {
        while let Some((msg, addr)) = self.recv_message().await {
            let is_remote_addr = self.remote_addrs.iter().any(|r| are_addrs_eq(r, &addr));
            if !is_remote_addr {
                self.queue.push_back((msg, addr));
                continue;
            }
            return Some(msg);
        }
        None
    }

    async fn send_message_to_client(&self, msg: &str, addr: &SocketAddr) {
        if let Err(error) = self.socket.send_to(msg.as_bytes(), addr).await {
            self.send_event(Event::from(event::MessageError {
                from_addr: Some(self.remote_addr()),
                local_addr: self.local_addr(),
                to_addr: *addr,
                error,
            }))
            .await;
        }
    }

    async fn send_message_to_remote(&self, msg: &str, client_addr: &SocketAddr) {
        if let Err(error) = self
            .socket
            .send_to(msg.as_bytes(), &*self.remote_addrs)
            .await
        {
            self.send_event(Event::from(event::MessageError {
                from_addr: Some(client_addr.clone()),
                local_addr: self.local_addr(),
                to_addr: self.remote_addr(),
                error,
            }))
            .await;
        }
    }

    fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }

    fn remote_addr(&self) -> SocketAddr {
        self.remote_addrs.first().unwrap().clone()
    }

    async fn send_event(&self, event: Event) {
        if let Some(event_sender) = &self.event_sender {
            let _ = event_sender.send(event).await;
        }
    }
}
