#![forbid(unsafe_code)]
#![allow(dead_code)] // TODO: remove this after stable

mod business;
mod connect;
mod context;
mod crypto;
mod error;
mod event;
mod key_store;
mod packet;
mod ping;
mod proto;
mod session;
mod sign;
mod socket;
mod tlv;

use std::sync::Arc;

use bytes::Bytes;
pub use error::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::business::{Business, BusinessHandle};
use crate::connect::optimum_server;
use crate::context::{AppInfo, Context, DeviceInfo};
use crate::event::downcast_event;
use crate::event::trans_emp::{TransEmp, TransEmp31};
pub use crate::key_store::KeyStore;
use crate::session::{QrSign, Session};
use crate::sign::{default_sign_provider, SignProvider};

/// The Protocol for the client
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Protocol {
    Windows = 0,
    MacOS = 1,
    Linux = 2,
}

/// Configuration for the client
pub struct ClientConfig {
    /// The protocol for the client, default is Linux
    pub protocol: Protocol,
    /// Auto reconnect to server when disconnected
    pub auto_reconnect: bool,
    /// Use the IPv6 to connect to server, only if your network support IPv6
    pub use_ipv6_network: bool,
    /// Get optimum server from Tencent MSF server, set to false to use hardcode server
    pub get_optimum_server: bool,
    /// Custom Sign Provider
    pub sign_provider: Option<Box<dyn SignProvider>>,
    ///  The maximum size of the highway block in byte, max 1MB (1024 * 1024 byte)
    pub highway_chuck_size: usize,
    /// Highway uploading concurrency, if the image failed to send, set this to 1
    pub highway_concurrency: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            protocol: Protocol::Linux,
            auto_reconnect: true,
            use_ipv6_network: false,
            get_optimum_server: true,
            sign_provider: None,
            highway_chuck_size: 1024 * 1024,
            highway_concurrency: 4,
        }
    }
}

pub struct Client {
    business: Business,
    handle: ClientHandle,
}

impl Client {
    pub async fn new(
        config: ClientConfig,
        device: DeviceInfo,
        key_store: KeyStore,
    ) -> Result<Self> {
        let app_info = AppInfo::get(config.protocol);
        let context = Context {
            app_info,
            device,
            key_store,
            sign_provider: config
                .sign_provider
                .unwrap_or_else(|| default_sign_provider(config.protocol)),
            crypto: Default::default(),
            session: Session::new(),
        };
        let context = Arc::new(context);

        let addr = optimum_server(config.get_optimum_server, config.use_ipv6_network).await?;
        let business = Business::new(addr, context.clone()).await?;

        let handle = ClientHandle {
            business: business.handle(),
            context,
        };

        Ok(Self { business, handle })
    }

    pub fn handle(&self) -> ClientHandle {
        self.handle.clone()
    }

    pub async fn spawn(&mut self) {
        // TODO: event stream
        self.business.spawn().await;
    }
}

#[derive(Clone)]
pub struct ClientHandle {
    business: Arc<BusinessHandle>,
    context: Arc<Context>,
}

impl ClientHandle {
    pub async fn fetch_qrcode(&self) -> Result<(String, Bytes)> {
        let trans_emp = TransEmp::FetchQrCode;
        let response = self.business.send_event(&trans_emp).await?;
        let event: &TransEmp31 = response
            .first()
            .and_then(downcast_event)
            .ok_or(Error::InvalidServerResponse("parsing error".into()))?;

        let qr_sign = QrSign {
            sign: event
                .signature
                .as_ref()
                .try_into()
                .map_err(|_| Error::InvalidServerResponse("invalid QR signature".into()))?,
            string: event.qr_sig.clone(),
            url: event.url.clone(),
        };

        self.context.session.qr_sign.store(Some(Arc::new(qr_sign)));

        tracing::info!("QR code fetched, expires in {} seconds", event.expiration);

        Ok((event.url.clone(), event.qr_code.clone()))
    }
}
