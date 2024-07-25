use proxy_rs::tcp::Proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let proxy = Proxy::builder()
        .local_addrs("127.0.0.1:8000")
        .remote_addrs("127.0.0.1:3000")
        .event_sender(tx)
        .build()
        .await?;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            println!("{:?}", event);
        }
    });

    proxy.run().await;

    Ok(())
}
