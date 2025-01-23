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
use crate::event::alive::Alive;
use crate::event::downcast_event;
use crate::event::info_sync::InfoSync;
use crate::event::trans_emp::{NTLoginHttpRequest, NTLoginHttpResponse, TransEmp, TransEmp12Res};
use crate::event::wtlogin::WtLogin;
pub use crate::key_store::KeyStore;
use crate::session::{QrSign, Session};
use crate::sign::{default_sign_provider, SignProvider};
use bytes::Bytes;
pub use error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::env;
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
        // TODO: (non-internal) event stream
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
        let mut trans_emp = TransEmp::new_fetch_qr_code();
        let response = self.business.send_event(&mut trans_emp).await?;
        let event: &TransEmp = downcast_event(&response).unwrap();
        let result = event.emp31_result.as_ref().unwrap();
        let qr_sign = QrSign {
            sign: result
                .signature
                .as_ref()
                .try_into()
                .map_err(|_| Error::InvalidServerResponse("invalid QR signature".into()))?,
            string: result.qr_sig.clone(),
            url: result.url.clone(),
        };

        self.context.session.qr_sign.store(Some(Arc::from(qr_sign)));
        tracing::info!("QR code fetched, expires in {} seconds", result.expiration);

        Ok((result.url.clone(), result.qr_code.clone()))
    }

    async fn query_trans_tmp_status(&self) -> Result<TransEmp12Res> {
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
            let mut query_result = TransEmp::new_query_result();
            let res = self.business.send_event(&mut query_result).await?;
            let res: &TransEmp = downcast_event(&res).unwrap();
            let result = res.emp12_result.as_ref().unwrap();
            Ok(result.to_owned())
        } else {
            Err(Error::GenericError("QR code not fetched".into()))
        }
    }

    async fn do_wt_login(&self) -> Result<()> {
        let res = self.business.send_event(&mut WtLogin::default()).await?;
        let event: &WtLogin = downcast_event(&res).unwrap();
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
                    TransEmp12Res::WaitingForScan => {
                        tracing::info!("Waiting for scan...");
                    }
                    TransEmp12Res::WaitingForConfirm => {
                        tracing::info!("Waiting for confirm...");
                    }
                    TransEmp12Res::Confirmed(data) => {
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
                    TransEmp12Res::CodeExpired => {
                        return Err(Error::GenericError("QR code expired".into()));
                    }
                    TransEmp12Res::Canceled => {
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
        let res = self.business.send_event(&mut InfoSync).await?;
        let _: &InfoSync = downcast_event(&res).unwrap();

        let (tx, mut rx) = watch::channel::<()>(());
        let handle = self.business.clone();

        let heartbeat = async move {
            const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let res = handle.send_event(&mut Alive).await.unwrap();
                        let _: &Alive = downcast_event(&res).unwrap();
                    }
                    _ = rx.changed() => break,
                }
            }
        };
        tokio::spawn(heartbeat);
        Ok(tx)
    }
}
