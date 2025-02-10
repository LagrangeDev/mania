use mania::{Client, ClientConfig, DeviceInfo, KeyStore};
use std::fs;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "tokio-tracing")] {
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
        } else {
            tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new("debug"))
            .init();
        }
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
    let mut event_listener = operator.event_listener.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = event_listener.system.changed() => {
                    if let Some(ref be) = *event_listener.system.borrow() {
                        tracing::info!("[SystemEvent] {:?}", be);
                    }
                }
                _ = event_listener.friend.changed() => {
                    if let Some(ref fe) = *event_listener.friend.borrow() {
                        tracing::info!("[FriendEvent] {:?}", fe);
                    }
                }
                _ = event_listener.group.changed() => {
                    if let Some(ref ge) = *event_listener.group.borrow() {
                        tracing::info!("[GroupEvent] {:?}", ge);
                    }
                }
            }
        }
    });
    tokio::spawn(async move {
        client.spawn().await;
    });
    if need_login {
        tracing::warn!("Session is invalid, need to login again!");
        let login_res: Result<(), String> = async {
            let (url, bytes) = operator.fetch_qrcode().await.map_err(|e| e.to_string())?;
            let qr_code_name = format!("qrcode_{}.png", Uuid::new_v4());
            fs::write(&qr_code_name, &bytes).map_err(|e| e.to_string())?;
            tracing::info!(
                "QR code fetched successfully! url: {}, saved to {}",
                url,
                qr_code_name
            );
            let login_res = operator.login_by_qrcode().await.map_err(|e| e.to_string());
            match fs::remove_file(&qr_code_name).map_err(|e| e.to_string()) {
                Ok(_) => tracing::info!("QR code file {} deleted successfully", qr_code_name),
                Err(e) => tracing::error!("Failed to delete QR code file {}: {}", qr_code_name, e),
            }
            login_res
        }
        .await;
        if let Err(e) = login_res {
            panic!("Failed to login: {:?}", e);
        }
    } else {
        tracing::info!("Session is still valid, trying to online...");
    }
    let _tx = match operator.online().await {
        Ok(tx) => tx,
        Err(e) => {
            panic!("Failed to set online status: {:?}", e);
        }
    };
    operator
        .update_key_store()
        .save("keystore.json")
        .unwrap_or_else(|e| tracing::error!("Failed to save key store: {:?}", e));
    tokio::signal::ctrl_c().await.unwrap();
}
