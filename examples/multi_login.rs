use mania::event::group::GroupEvent;
use mania::message::builder::MessageChainBuilder;
use mania::{Client, ClientConfig, DeviceInfo, KeyStore};
use std::fs;
use std::io::stdout;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "tokio-tracing")] {
            let console_layer = console_subscriber::spawn();
            tracing_subscriber::registry()
                .with(console_layer)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_filter(tracing_subscriber::EnvFilter::new("trace")),
                )
                .init();
            tracing::info!("tokio-tracing initialized.");
        } else {
            use tracing_subscriber::{fmt, EnvFilter};
            use tracing_appender::rolling::{RollingFileAppender, Rotation};
            let file_appender = RollingFileAppender::new(Rotation::HOURLY, "./logs", "mania.log");
            let fmt_layer = fmt::Layer::default()
                .with_writer(stdout)
                .with_filter(EnvFilter::new("debug"));
            let file_layer = fmt::Layer::default()
                .with_writer(file_appender)
                .with_filter(EnvFilter::new("trace"));
            let subscriber = tracing_subscriber::registry()
                .with(fmt_layer)
                .with(file_layer);
            subscriber.init();
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

    let op = client.handle().operator().clone();
    let send_op = client.handle().operator().clone();
    let mut group_receiver = op.event_listener.group.clone();
    let mut system_receiver = op.event_listener.system.clone();
    let mut friend_receiver = op.event_listener.friend.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = system_receiver.changed() => {
                    if let Some(ref se) = *system_receiver.borrow() {
                        tracing::info!("[SystemEvent] {:?}", se);
                    }
                }
                _ = friend_receiver.changed() => {
                    if let Some(ref fe) = *friend_receiver.borrow() {
                        tracing::info!("[FriendEvent] {:?}", fe);
                    }
                }
                _ = group_receiver.changed() => {
                    let maybe_data = {
                        let guard = group_receiver.borrow();
                        if let Some(ref ge) = *guard {
                            tracing::info!("[GroupEvent] {:?}", ge);
                            match ge {
                                GroupEvent::GroupMessage(gme) => {
                                    if let mania::message::chain::MessageType::Group(gmeu) = &gme.chain.typ {
                                        let chain_str = gme.chain.to_string();
                                        Some((chain_str, gmeu.group_uin))
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    };
                    if let Some((chain_str, group_uin)) = maybe_data {
                        if chain_str.contains("/mania ping") {
                            let chain = MessageChainBuilder::group(group_uin)
                                .text("pong")
                                .build();
                            send_op.send_message(chain).await.unwrap();
                        }
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
            let (url, bytes) = op.fetch_qrcode().await.map_err(|e| e.to_string())?;
            let qr_code_name = format!("qrcode_{}.png", Uuid::new_v4());
            fs::write(&qr_code_name, &bytes).map_err(|e| e.to_string())?;
            tracing::info!(
                "QR code fetched successfully! url: {}, saved to {}",
                url,
                qr_code_name
            );
            let login_res = op.login_by_qrcode().await.map_err(|e| e.to_string());
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

    let _tx = match op.online().await {
        Ok(tx) => tx,
        Err(e) => {
            panic!("Failed to set online status: {:?}", e);
        }
    };

    op.update_key_store()
        .save("keystore.json")
        .unwrap_or_else(|e| tracing::error!("Failed to save key store: {:?}", e));

    tokio::signal::ctrl_c().await.unwrap();
}
