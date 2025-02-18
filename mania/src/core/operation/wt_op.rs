use crate::core::business::BusinessHandle;
use crate::core::event::downcast_major_event;
use crate::core::event::login::trans_emp::{
    NTLoginHttpRequest, NTLoginHttpResponse, TransEmp, TransEmp12Res, TransEmpResult,
};
use crate::core::event::login::wtlogin::WtLogin;
use crate::core::event::system::alive::AliveEvent;
use crate::core::event::system::info_sync::InfoSyncEvent;
use crate::core::event::system::nt_sso_alive::NtSsoAliveEvent;
use crate::core::http;
use crate::core::session::QrSign;
use crate::event::system::SystemEvent;
use crate::event::system::bot_online::BotOnlineEvent;
use crate::{KeyStore, ManiaError, ManiaResult};
use bytes::Bytes;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{sleep, timeout};

impl BusinessHandle {
    pub fn update_key_store(&self) -> &KeyStore {
        &self.context.key_store
    }

    pub async fn fetch_qrcode(self: &Arc<Self>) -> ManiaResult<(String, Bytes)> {
        let mut trans_emp = TransEmp::new_fetch_qr_code();
        let response = self.send_event(&mut trans_emp).await?;
        let event: &TransEmp =
            downcast_major_event(&response).ok_or(ManiaError::InternalEventDowncastError)?;
        let result = event
            .result
            .as_ref()
            .ok_or_else(|| ManiaError::GenericError("Emp result not found".into()))?;
        if let TransEmpResult::Emp31(emp31) = result {
            let qr_sign = QrSign {
                sign: emp31
                    .signature
                    .as_ref()
                    .try_into()
                    .map_err(|_| ManiaError::GenericError("invalid QR signature".into()))?,
                string: emp31.qr_sig.clone(),
                url: emp31.url.clone(),
            };
            self.context.session.qr_sign.store(Some(Arc::from(qr_sign)));
            tracing::info!("QR code fetched, expires in {} seconds", emp31.expiration);
            Ok((emp31.url.clone(), emp31.qr_code.clone()))
        } else {
            panic!("Emp31 not found in response");
        }
    }

    async fn query_trans_tmp_status(self: &Arc<Self>) -> ManiaResult<TransEmp12Res> {
        if let Some(qr_sign) = (*self.context.session.qr_sign.load()).clone() {
            let request_body = NTLoginHttpRequest {
                appid: self.context.app_info.app_id as u64,
                qrsig: qr_sign.string.clone(),
                face_update_time: 0,
            };
            let payload = serde_json::to_vec(&request_body).map_err(|e| {
                ManiaError::GenericError(Cow::from(format!(
                    "Failed to serialize request body: {:?}",
                    e
                )))
            })?;
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            );
            let response = http::client()
                .post_binary_async("https://ntlogin.qq.com/qr/getFace", &payload, Some(headers))
                .await
                .map_err(|e| {
                    ManiaError::GenericError(Cow::from(format!(
                        "Failed to query QR code status via ntlogin.qq.com: {:?}",
                        e
                    )))
                })?;
            let info: NTLoginHttpResponse = serde_json::from_slice(&response).map_err(|e| {
                ManiaError::GenericError(Cow::from(format!(
                    "Failed to deserialize response: {:?}",
                    e
                )))
            })?;
            self.context.key_store.uin.store(info.uin.into());
            let mut query_result = TransEmp::new_query_result();
            let res = self.send_event(&mut query_result).await?;
            let res: &TransEmp =
                downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
            let result = res
                .result
                .as_ref()
                .ok_or_else(|| ManiaError::GenericError("Emp result not found".into()))?;
            if let TransEmpResult::Emp12(emp12) = result {
                Ok(emp12.to_owned())
            } else {
                panic!("Emp12 not found in response");
            }
        } else {
            Err(ManiaError::GenericError("QR code not fetched".into()))
        }
    }

    async fn do_wt_login(self: &Arc<Self>) -> ManiaResult<()> {
        let res = self.send_event(&mut WtLogin::default()).await?;
        let event: &WtLogin =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        match event.code {
            0 => {
                tracing::info!(
                    "WTLogin success, welcome {:?} ヾ(≧▽≦*)o",
                    self.context.key_store.info
                );
                Ok(())
            }
            _ => Err(ManiaError::GenericError(
                format!(
                    "WTLogin failed with code: {}, msg: {:?} w(ﾟДﾟ)w",
                    event.code, event.msg
                )
                .into(),
            )),
        }
    }

    pub async fn login_by_qrcode(self: &Arc<Self>) -> ManiaResult<()> {
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
                        return Err(ManiaError::GenericError("QR code expired".into()));
                    }
                    TransEmp12Res::Canceled => {
                        return Err(ManiaError::GenericError(
                            "QR code login canceled by user".into(),
                        ));
                    }
                }
                sleep(interval).await;
            }
        })
        .await;
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ManiaError::GenericError(
                "QR code scan timed out after 120s!".into(),
            )),
        }
    }

    pub async fn online(self: &Arc<Self>) -> ManiaResult<watch::Sender<()>> {
        let (tx, mut rx) = watch::channel::<()>(());
        let res = self.send_event(&mut InfoSyncEvent).await?;
        let _: &InfoSyncEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        tracing::info!("Online success");
        tracing::debug!(
            "d2key: {:?}",
            hex::encode(**self.context.key_store.session.d2_key.load())
        );
        self.event_dispatcher
            .system
            .send(Some(SystemEvent::BotOnlineEvent(BotOnlineEvent {
                reason: None,
            })))
            .expect("send BotOnlineEvent failed");
        let handle = self.clone();
        let heartbeat = async move {
            let mut hb_interval = tokio::time::interval(Duration::from_secs(10));
            let mut nt_hb_interval = tokio::time::interval(Duration::from_secs(270));
            loop {
                tokio::select! {
                    _ = hb_interval.tick() => {
                        if let Err(e) = handle.push_event(&AliveEvent).await {
                            tracing::error!("Failed to send Alive event: {:?}", e);
                        }
                    }
                    _ = nt_hb_interval.tick() => {
                        if let Err(e) = handle.push_event(&NtSsoAliveEvent).await {
                            tracing::error!("Failed to send NtSsoAlive event: {:?}", e);
                        }
                    }
                    _ = rx.changed() => break,
                }
            }
        };
        tokio::spawn(heartbeat);
        Ok(tx)
    }
}
