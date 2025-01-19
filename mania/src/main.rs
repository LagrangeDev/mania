use mania::*;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("debug"))
        .init();
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
    let need_login = key_store.is_session_expired().await
        || key_store.session.d2.read().await.is_empty()
        || key_store.session.tgt.read().await.is_empty();
    let mut client = Client::new(config, device, key_store).await.unwrap();
    let client_handle = client.handle();
    tokio::spawn(async move {
        client.spawn().await;
    });
    if need_login {
        tracing::info!("Session is invalid, need to login again!");
        match client_handle.fetch_qrcode().await {
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
        client_handle
            .login_by_qrcode()
            .await
            .expect("Failed to login by QR code");
    } else {
        tracing::info!("Session is still valid, no need to login again.");
    }
    let _tx = client_handle.online().await.unwrap();
    client_handle
        .update_key_store()
        .save("keystore.json")
        .unwrap_or_else(|e| tracing::error!("Failed to save key store: {:?}", e));
    tokio::signal::ctrl_c().await.unwrap();
}
