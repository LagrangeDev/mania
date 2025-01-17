use mania::context::DeviceInfo;
use mania::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = ClientConfig::default();
    let device = DeviceInfo::default();
    let key_store = KeyStore::default();
    let mut client = Client::new(config, device, key_store).await.unwrap();
    let client_handle = client.handle();
    tokio::spawn(async move {
        client.spawn().await;
    });
    match client_handle.fetch_qrcode().await {
        Ok((url, _)) => tracing::info!("QR code fetched successfully! url: {}", url),
        Err(e) => tracing::error!("Failed to fetch QR code: {:?}", e),
    }
}
