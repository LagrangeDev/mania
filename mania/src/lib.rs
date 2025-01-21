#![forbid(unsafe_code)]
#![allow(dead_code)] // TODO: remove this after stable

mod business;
mod connect;
pub mod context;
mod crypto;
mod error;
mod event;
mod http;
mod key_store;
mod packet;
mod ping;
mod proto;
mod session;
mod sign;
mod socket;
mod tlv;

use crate::business::{Business, BusinessHandle};
use crate::connect::optimum_server;
pub use crate::context::{AppInfo, Context, DeviceInfo};
use crate::event::alive::{Alive, AliveRes};
use crate::event::downcast_event;
use crate::event::info_sync::{InfoSync, InfoSyncRes};
pub use crate::event::trans_emp::{NTLoginHttpRequest, TransEmp, TransEmp31};
use crate::event::trans_emp::{NTLoginHttpResponse, TransEmp12};
use crate::event::wtlogin::{WtLogin, WtLoginRes};
pub use crate::key_store::KeyStore;
use crate::session::{QrSign, Session};
use crate::sign::{default_sign_provider, SignProvider};
use bytes::Bytes;
pub use error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::env;
pub use std::io::Write;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{sleep, timeout, Duration};

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
            sign_provider: config.sign_provider.unwrap_or_else(|| {
                default_sign_provider(
                    config.protocol,
                    env::var("MANIA_LINUX_SIGN_URL")
                        .ok()
                        .map(Some)
                        .unwrap_or_else(|| {
                            tracing::warn!("MANIA_LINUX_SIGN_URL not set, login maybe fail!");
                            None
                        }),
                )
            }),
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
    pub fn update_key_store(&self) -> &KeyStore {
        &self.context.key_store
    }

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

        self.context.session.qr_sign.store(Some(Arc::from(qr_sign)));
        tracing::info!("QR code fetched, expires in {} seconds", event.expiration);

        Ok((event.url.clone(), event.qr_code.clone()))
    }

    async fn query_trans_tmp_status(&self) -> Result<TransEmp12> {
        if let Some(qr_sign) = &*self.context.session.qr_sign.load() {
            let request_body = NTLoginHttpRequest {
                appid: self.context.app_info.app_id as u64,
                qrsig: qr_sign.string.clone(),
                face_update_time: 0,
            };
            let payload = serde_json::to_vec(&request_body).unwrap();
            let response = http::client()
                .post_binary_async(
                    "https://ntlogin.qq.com/qr/getFace",
                    &payload,
                    "application/json",
                )
                .await
                .unwrap();
            let info: NTLoginHttpResponse = serde_json::from_slice(&response).unwrap();
            self.context.key_store.uin.store(info.uin.into());
            let res = self.business.send_event(&TransEmp::QueryResult).await?;
            let res: &TransEmp12 = res.first().and_then(downcast_event).unwrap();
            Ok(res.to_owned())
        } else {
            Err(Error::GenericError("QR code not fetched".into()))
        }
    }

    async fn do_wt_login(&self) -> Result<()> {
        let res = self.business.send_event(&WtLogin {}).await?;
        let event: &WtLoginRes = res.first().and_then(downcast_event).unwrap();
        match event.code {
            0 => {
                tracing::info!(
                    "WTLogin success, welcome {:?} ヾ(≧▽≦*)o",
                    self.context.key_store.info
                );
                Ok(())
            }
            _ => Err(Error::GenericError(
                format!(
                    "WTLogin failed with code: {}, msg: {:?} w(ﾟДﾟ)w",
                    event.code, event.msg
                )
                .into(),
            )),
        }
    }

    pub async fn login_by_qrcode(&self) -> Result<()> {
        let interval = Duration::from_secs(2);
        let timeout_duration = Duration::from_secs(120);
        let result = timeout(timeout_duration, async {
            loop {
                let status = match self.query_trans_tmp_status().await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("query_trans_tmp_status failed: {:?}", e);
                        return Err(e);
                    }
                };
                match status {
                    TransEmp12::WaitingForScan => {
                        tracing::info!("Waiting for scan...");
                    }
                    TransEmp12::WaitingForConfirm => {
                        tracing::info!("Waiting for confirm...");
                    }
                    TransEmp12::Confirmed(data) => {
                        tracing::info!("QR code confirmed, logging in...");
                        self.context
                            .session
                            .stub
                            .tgtgt_key
                            .store(Arc::from(data.tgtgt_key));
                        self.context
                            .key_store
                            .session
                            .temp_password
                            .store(Some(Arc::from(data.temp_password)));
                        self.context
                            .key_store
                            .session
                            .no_pic_sig
                            .store(Some(Arc::from(data.no_pic_sig)));
                        return self.do_wt_login().await;
                    }
                    TransEmp12::CodeExpired => {
                        return Err(Error::GenericError("QR code expired".into()));
                    }
                    TransEmp12::Canceled => {
                        return Err(Error::GenericError("QR code login canceled by user".into()));
                    }
                }
                sleep(interval).await;
            }
        })
        .await;
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::GenericError(
                "QR code scan timed out after 120s!".into(),
            )),
        }
    }

    pub async fn online(&self) -> Result<watch::Sender<()>> {
        let res = self.business.send_event(&InfoSync).await?;
        let _: &InfoSyncRes =
            res.first()
                .and_then(downcast_event)
                .ok_or(Error::InvalidServerResponse(
                    "parsing InfoSyncRes error".into(),
                ))?; // TODO: parse InfoSyncRes

        let (tx, mut rx) = watch::channel::<()>(());
        let handle = self.business.clone();

        let heartbeat = async move {
            const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let res = handle.send_event(&Alive).await.unwrap();
                        let _: &AliveRes = res.first().and_then(downcast_event).unwrap();
                    }
                    _ = rx.changed() => break,
                }
            }
        };
        tokio::spawn(heartbeat);
        Ok(tx)
    }
}
