use crate::core::http;
use crate::core::sign::{SignProvider, SignResult};
use bytes::Bytes;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SignServerReq {
    cmd: String,
    seq: u32,
    src: String,
}

#[derive(Deserialize)]
struct SignServerResp {
    value: SignResult,
    platform: String,
    version: String,
}

pub struct LinuxSignProvider {
    pub url: Option<String>,
}

impl SignProvider for LinuxSignProvider {
    fn sign_impl(&self, cmd: &str, seq: u32, body: &[u8]) -> Option<SignResult> {
        let dummy_sign = || -> SignResult {
            SignResult {
                sign: Bytes::from(&[0u8; 20][..]),
                extra: Bytes::new(),
                token: String::new(),
            }
        };
        match self.url.as_ref() {
            Some(url) => {
                let request_body = SignServerReq {
                    cmd: cmd.to_string(),
                    seq,
                    src: hex::encode(body),
                };
                let payload = match serde_json::to_vec(&request_body) {
                    Ok(payload) => payload,
                    Err(e) => {
                        tracing::error!("failed to serialize SignServerReq: {}", e);
                        return Some(dummy_sign());
                    }
                };
                let response = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let mut headers = HeaderMap::new();
                        headers.insert("Content-Type", "application/json".parse().unwrap());
                        http::client()
                            .post_binary_async(url.as_str(), &payload, Some(headers))
                            .await
                    })
                });
                let resp: Option<SignServerResp> = match response {
                    Ok(resp) => match serde_json::from_slice(&resp) {
                        Ok(resp) => Some(resp),
                        Err(e) => {
                            tracing::error!("failed to deserialize SignServerResp: {}", e);
                            None
                        }
                    },
                    Err(e) => {
                        tracing::error!("failed to send request to sign server: {}", e);
                        None
                    }
                };
                resp.map(|r| r.value).or_else(|| Some(dummy_sign()))
            }
            None => {
                tracing::warn!("sign server url is not set, using dummy sign");
                Some(dummy_sign())
            }
        }
    }
}
