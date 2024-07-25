use proxy_rs::udp::Proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proxy = Proxy::builder()
        .local_addrs(":::8000")
        .remote_addrs("127.0.0.1:3000")
        .build()
        .await?;

    proxy.run().await;

    Ok(())
}
