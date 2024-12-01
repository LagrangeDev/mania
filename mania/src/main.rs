use mania::*;
use mania::context::DeviceInfo;

#[tokio::main]
async fn main() {
    let config = ClientConfig::default();
    let device = DeviceInfo::default();
    let key_store = KeyStore::default();
    let mut client = Client::new(config, device, key_store).await.unwrap();

    let client_handle = client.handle();

    tokio::spawn(async move {
        client.spawn().await;
    });

    match client_handle.fetch_qrcode().await {
        Ok((url, _)) => println!("QR code fetched successfully! url: {}", url),
        Err(e) => eprintln!("Failed to fetch QR code: {:?}", e),
    }
}