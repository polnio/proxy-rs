pub mod tcp;
pub mod udp;

mod builder;
mod event;

pub use builder::ProxyBuilder;
pub use event::Event;
use tokio::sync::mpsc;

#[derive(Debug)]
struct ProxyEventManager {
    event_sender: mpsc::Sender<Event>,
    tcp_event_receiver: mpsc::Receiver<tcp::Event>,
    udp_event_receiver: mpsc::Receiver<udp::Event>,
}

impl ProxyEventManager {
    pub async fn run(mut self, mut close_rx: mpsc::Receiver<()>) {
        loop {
            tokio::select! {
                Some(event) = self.tcp_event_receiver.recv() => {
                    let _ = self.event_sender.send(Event::Tcp(event)).await;
                }
                Some(event) = self.udp_event_receiver.recv() => {
                    let _ = self.event_sender.send(Event::Udp(event)).await;
                }
                _ = close_rx.recv() => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Proxy {
    tcp: tcp::Proxy,
    udp: udp::Proxy,
    event_manager: Option<ProxyEventManager>,
}

impl Proxy {
    pub fn builder() -> ProxyBuilder {
        ProxyBuilder::new()
    }

    pub async fn run(self) {
        let (close_tx, close_rx) = mpsc::channel(1);

        if let Some(event_manager) = self.event_manager {
            tokio::spawn(event_manager.run(close_rx));
        }

        let tcp = self.tcp.run();
        let udp = self.udp.run();
        tokio::select! {
            _ = tcp => {}
            _ = udp => {}
        }

        let _ = close_tx.send(()).await;
    }
}
