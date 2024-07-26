use super::{Event, Proxy};
use crate::{tcp, udp, ProxyEventManager};
use tokio::io;
use tokio::net::ToSocketAddrs;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ProxyBuilder {
    tcp: tcp::ProxyBuilder,
    udp: udp::ProxyBuilder,

    event_sender: Option<mpsc::Sender<Event>>,
}

impl ProxyBuilder {
    pub fn new() -> Self {
        Self {
            tcp: tcp::ProxyBuilder::new(),
            udp: udp::ProxyBuilder::new(),

            event_sender: None,
        }
    }
    pub fn local_addrs<A: ToSocketAddrs + Send + 'static + Clone>(
        mut self,
        local_addrs: A,
    ) -> Self {
        self.tcp = self.tcp.local_addrs(local_addrs.clone());
        self.udp = self.udp.local_addrs(local_addrs);
        self
    }

    pub fn remote_addrs<A: ToSocketAddrs + Send + 'static + Clone>(
        mut self,
        remote_addrs: A,
    ) -> Self {
        self.tcp = self.tcp.remote_addrs(remote_addrs.clone());
        self.udp = self.udp.remote_addrs(remote_addrs);
        self
    }

    pub fn event_sender(mut self, event_sender: mpsc::Sender<Event>) -> Self {
        self.event_sender = Some(event_sender.clone());
        self
    }

    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.tcp = self.tcp.buffer_size(buffer_size);
        self.udp = self.udp.buffer_size(buffer_size);
        self
    }

    pub async fn build(self) -> io::Result<Proxy> {
        if let Some(event_sender) = self.event_sender {
            let (tcp_event_sender, tcp_event_receiver) = mpsc::channel(event_sender.max_capacity());
            let (udp_event_sender, udp_event_receiver) = mpsc::channel(event_sender.max_capacity());
            let tcp = self.tcp.event_sender(tcp_event_sender).build().await?;
            let udp = self.udp.event_sender(udp_event_sender).build().await?;
            let event_manager = Some(ProxyEventManager {
                event_sender,
                tcp_event_receiver,
                udp_event_receiver,
            });
            Ok(Proxy {
                tcp,
                udp,
                event_manager,
            })
        } else {
            let tcp = self.tcp.build().await?;
            let udp = self.udp.build().await?;
            let event_manager = None;
            Ok(Proxy {
                tcp,
                udp,
                event_manager,
            })
        }
    }
}
