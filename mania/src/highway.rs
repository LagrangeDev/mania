pub mod hw_client;
mod hw_frame_codec;

use arc_swap::ArcSwap;
use bytes::Bytes;

use crate::highway::hw_client::HighwayClient;
use mania_core::core::protos::service::highway::{NtHighwayDomain, NtHighwayIPv4};
use mania_core::core::protos::service::oidb::IPv4;
use std::borrow::Cow;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HighwayError {
    #[error("Invalid frame!")]
    InvalidFrame,
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Upload error! code={0}")]
    UploadError(u32),
    #[error("Hex decode error!")]
    HexError(#[from] hex::FromHexError),
    #[error("Audio codec error: {0}")]
    AudioCodecError(#[from] mania_codec::audio::AudioCodecError),
    #[error("An error occurred in highway: {0}")]
    OtherError(Cow<'static, str>),
}

fn int32ip2str(ip: u32) -> String {
    let a = ip & 0xff;
    let b = (ip >> 8) & 0xff;
    let c = (ip >> 16) & 0xff;
    let d = (ip >> 24) & 0xff;
    format!("{a}.{b}.{c}.{d}")
}

pub fn oidb_ipv4s_to_highway_ipv4s(ipv4s: &[IPv4]) -> Vec<NtHighwayIPv4> {
    ipv4s
        .iter()
        .map(|ip| NtHighwayIPv4 {
            domain: Some(NtHighwayDomain {
                is_enable: true,
                ip: int32ip2str(ip.out_ip),
            }),
            port: ip.out_port,
        })
        .collect()
}

#[derive(Default)]
pub struct Highway {
    pub sig_session: ArcSwap<Option<Bytes>>,
    pub prepare_guard: tokio::sync::Mutex<()>,
    pub client: ArcSwap<HighwayClient>,
}
