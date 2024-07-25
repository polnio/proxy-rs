use super::{Event, Proxy};
use std::collections::VecDeque;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::{lookup_host, ToSocketAddrs, UdpSocket};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

#[derive(Debug)]
pub struct ProxyBuilder {
    local_addrs_handles: JoinSet<io::Result<Vec<SocketAddr>>>,
    remote_addrs_handles: JoinSet<io::Result<Vec<SocketAddr>>>,
    event_sender: Option<mpsc::Sender<Event>>,
    buffer_size: usize,
}

impl ProxyBuilder {
    pub fn new() -> Self {
        Self {
            local_addrs_handles: JoinSet::new(),
            remote_addrs_handles: JoinSet::new(),
            event_sender: None,
            buffer_size: 1024,
        }
    }
    pub fn local_addrs<A: ToSocketAddrs + Send + 'static>(mut self, local_addrs: A) -> Self {
        self.local_addrs_handles.spawn(async move {
            lookup_host(local_addrs)
                .await
                .map(|lookup_host| lookup_host.collect())
        });
        self
    }

    pub fn remote_addrs<A: ToSocketAddrs + Send + 'static>(mut self, remote_addrs: A) -> Self {
        self.remote_addrs_handles.spawn(async move {
            lookup_host(remote_addrs)
                .await
                .map(|lookup_host| lookup_host.collect())
        });
        self
    }

    pub fn event_sender(mut self, event_sender: mpsc::Sender<Event>) -> Self {
        self.event_sender = Some(event_sender);
        self
    }

    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    pub async fn build(mut self) -> io::Result<Proxy> {
        let mut local_addrs = Vec::new();
        while let Some(l) = self.local_addrs_handles.join_next().await {
            local_addrs.extend(l.unwrap()?);
        }
        let mut remote_addrs = Vec::new();
        while let Some(r) = self.remote_addrs_handles.join_next().await {
            remote_addrs.extend(r.unwrap()?);
        }
        let local_socket = UdpSocket::bind(&*local_addrs).await?;
        let event_sender = self.event_sender;
        let buffer_size = self.buffer_size;
        let queue = VecDeque::new();

        Ok(Proxy {
            socket: local_socket,
            remote_addrs,
            event_sender,
            buffer_size,
            queue,
        })
    }
}
