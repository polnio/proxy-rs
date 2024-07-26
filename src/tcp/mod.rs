mod builder;
mod event;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};

pub use self::builder::ProxyBuilder;
pub use self::event::Event;

#[derive(Debug)]
pub struct Proxy {
    remote_addrs: Vec<SocketAddr>,
    buffer_size: usize,

    listener: TcpListener,
    event_sender: Option<mpsc::Sender<Event>>,
}

impl Proxy {
    pub fn builder() -> ProxyBuilder {
        ProxyBuilder::new()
    }

    pub async fn run(self) {
        let this = Arc::new(self);
        loop {
            let Some((client_stream, client_addr)) = this.accept_client().await else {
                break;
            };
            let this = this.clone();
            tokio::spawn(async move {
                let Some((remote_stream, remote_addr)) = this.connect_remote().await else {
                    return;
                };

                this.send_event(Event::Connection {
                    client_addr: client_addr.clone(),
                    local_addr: this.local_addr(),
                    remote_addr: remote_addr.clone(),
                })
                .await;

                let (client_reader, client_writer) = client_stream.into_split();
                let (remote_reader, remote_writer) = remote_stream.into_split();
                let (close_sender, close_receiver) = broadcast::channel(2);

                let client_handle = tokio::spawn({
                    let this = this.clone();
                    let close_sender = close_sender.clone();
                    let close_receiver = close_sender.subscribe();
                    async move {
                        this.pipe(client_reader, remote_writer, close_receiver)
                            .await;
                        let _ = close_sender.send(());
                    }
                });
                let remote_handle = tokio::spawn({
                    let this = this.clone();
                    async move {
                        this.pipe(remote_reader, client_writer, close_receiver)
                            .await;
                        let _ = close_sender.send(());
                    }
                });

                client_handle.await.unwrap();
                remote_handle.await.unwrap();

                this.send_event(Event::Disconnection {
                    client_addr,
                    local_addr: this.local_addr(),
                    remote_addr,
                })
                .await;
            });
        }
    }

    async fn accept_client(&self) -> Option<(TcpStream, SocketAddr)> {
        match self.listener.accept().await {
            Ok(result) => Some(result),
            Err(error) => {
                let local_addr = self.local_addr();
                self.send_event(Event::ConnectionError { local_addr, error })
                    .await;
                None
            }
        }
    }

    async fn connect_remote(&self) -> Option<(TcpStream, SocketAddr)> {
        match TcpStream::connect(&*self.remote_addrs).await {
            Ok(result) => {
                let socket_addr = result.peer_addr().unwrap();
                Some((result, socket_addr))
            }
            Err(error) => {
                let local_addr = self.local_addr();
                self.send_event(Event::ConnectionError { local_addr, error })
                    .await;
                None
            }
        }
    }

    async fn pipe(
        &self,
        mut reader: OwnedReadHalf,
        mut writer: OwnedWriteHalf,
        mut close_receiver: broadcast::Receiver<()>,
    ) {
        let mut buffer = vec![0; self.buffer_size];
        let from_addr = reader.peer_addr().unwrap();
        let local_addr = self.local_addr();
        let to_addr = writer.peer_addr().unwrap();
        loop {
            tokio::select! {
                n = reader.read(&mut buffer) => {
                    let n = match n {
                        Ok(n) => n,
                        Err(error) => {
                            let local_addr = self.local_addr();
                            self.send_event(Event::MessageError {
                                from_addr,
                                local_addr,
                                to_addr,
                                error,
                            })
                            .await;
                            break;
                        }
                    };
                    if n == 0 {
                        break;
                    }
                    let message = String::from_utf8_lossy(&buffer[0..n]).to_string();
                    let _ = writer.write(&buffer[0..n]).await;
                    self.send_event(Event::Message {
                        from_addr,
                        local_addr,
                        to_addr,
                        message,
                    })
                    .await;
                }
                _ = close_receiver.recv() => {
                    break;
                }
            }
        }
    }

    fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    async fn send_event(&self, event: Event) {
        if let Some(event_sender) = &self.event_sender {
            let _ = event_sender.send(event).await;
        }
    }
}
