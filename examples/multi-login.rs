use mania::{Client, ClientConfig, DeviceInfo, KeyStore};
use std::io::Write;

#[tokio::main]
async fn main() {
    #[cfg(feature = "tokio-tracing")]
    {
        use tracing_subscriber::prelude::*;
        let console_layer = console_subscriber::spawn();
        tracing_subscriber::registry()
            .with(console_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::EnvFilter::new("debug")),
            )
            .init();
        tracing::info!("tokio-tracing initialized.");
    }
    #[cfg(not(feature = "tokio-tracing"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
            .init();
    }
    let config = ClientConfig::default();
    let device = DeviceInfo::load("device.json").unwrap_or_else(|_| {
        tracing::warn!("Failed to load device info, generating a new one...");
        let device = DeviceInfo::default();
        device.save("device.json").unwrap();
        device
    });
    let key_store = KeyStore::load("keystore.json").unwrap_or_else(|_| {
        tracing::warn!("Failed to load keystore, generating a new one...");
        let key_store = KeyStore::default();
        key_store.save("keystore.json").unwrap();
        key_store
    });
    let need_login = key_store.is_expired();
    let mut client = Client::new(config, device, key_store).await.unwrap();
    let operator = client.handle().operator();
    tokio::spawn(async move {
        client.spawn().await;
    });
    if need_login {
        tracing::info!("Session is invalid, need to login again!");
        match operator.fetch_qrcode().await {
            Ok((url, bytes)) => {
                tracing::info!(
                    "QR code fetched successfully! url: {}, also saved to qr.png",
                    url
                );
                let mut file = std::fs::File::create("qrcode.png").unwrap();
                file.write_all(&bytes).expect(
                    "Failed to write QR code image to file, please check the current directory.",
                )
            }
            Err(e) => tracing::error!("Failed to fetch QR code: {:?}", e),
        }
        operator
            .login_by_qrcode()
            .await
            .expect("Failed to login by QR code");
    } else {
        tracing::info!("Session is still valid, no need to login again.");
    }
    let _tx = operator.online().await.unwrap();
    operator
        .update_key_store()
        .save("keystore.json")
        .unwrap_or_else(|e| tracing::error!("Failed to save key store: {:?}", e));
    tokio::signal::ctrl_c().await.unwrap();
}
