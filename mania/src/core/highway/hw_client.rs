use crate::core::connect::tcp_connect_timeout;
use crate::core::highway::hw_frame_codec::{HighwayFrame, HighwayFrameCodec};
use crate::core::highway::{AsyncPureStream, HighwayError};
use crate::core::http;
use crate::core::protos::service::highway::{
    DataHighwayHead, LoginSigHead, ReqDataHighwayHead, RespDataHighwayHead, SegHead,
};
use crate::dda;
use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use futures::StreamExt;
use md5::{Digest, Md5};
use prost::Message;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::borrow::Cow;
use std::fmt::Debug;
use std::net::ToSocketAddrs;
use std::time::Duration;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct HighwaySession {
    pub ticket: Bytes,
    pub uin: String,
    pub cmd: u32,
    pub command: String,
    pub file_md5: Bytes,
    pub file_size: u32,
    pub ext: Bytes,
}

impl Debug for HighwaySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HighwaySession")
            .field("ticket", &hex::encode(self.ticket.clone()))
            .field("uin", &self.uin)
            .field("cmd", &self.cmd)
            .field("command", &self.command)
            .field("file_md5", &hex::encode(self.file_md5.clone()))
            .field("file_size", &self.file_size)
            .field("ext", &hex::encode(self.ext.clone()))
            .finish()
    }
}

impl HighwaySession {
    fn build_pic_up_head(&self, offset: u32, body_length: u32, body_md5: Vec<u8>) -> Bytes {
        ReqDataHighwayHead {
            msg_base_head: Some(dda!(DataHighwayHead {
                version: 1,
                uin: Some(self.uin.to_string()),
                command: Some(self.command.to_string()),
                seq: Some(0),
                retry_times: Some(0),
                app_id: 1600001604,
                data_flag: 16,
                command_id: self.cmd,
            })),
            msg_seg_head: Some(dda!(SegHead {
                service_id: Some(0),
                filesize: self.file_size as u64,
                data_offset: Some(offset as u64),
                data_length: body_length,
                service_ticket: self.ticket.to_vec(),
                md5: body_md5,
                file_md5: self.file_md5.to_vec(),
                cache_addr: Some(0),
                cache_port: Some(0),
            })),
            bytes_req_extend_info: Some(self.ext.to_vec()),
            timestamp: 0,
            msg_login_sig_head: Some(dda!(LoginSigHead {
                uint32_login_sig_type: 8,
                app_id: 1600001604,
            })),
        }
        .encode_to_vec()
        .into()
    }
}

#[derive(Default)]
pub struct HighwayClient {
    pub addr: String,
    pub uin: String,
    pub timeout: u32,
    pub ticket: Bytes,
    pub chunk_size: usize,
}

impl HighwayClient {
    pub fn new(address: &str, timeout: u32, ticket: Bytes, uin: u32, chunk_size: usize) -> Self {
        Self {
            addr: address.to_string(),
            timeout,
            ticket,
            uin: uin.to_string(),
            chunk_size,
        }
    }

    async fn read_stream<S>(&self, stream: &mut S, chunk_size: usize) -> io::Result<Vec<u8>>
    where
        S: AsyncRead + Unpin + ?Sized,
    {
        let mut limited = stream.take(chunk_size as u64);
        let mut buf = Vec::with_capacity(chunk_size);
        limited.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn build_frame(
        &self,
        cmd: u32,
        data: &mut AsyncPureStream,
        size: u32,
        md5: Bytes,
        ext_info: Bytes,
        offset: u32,
    ) -> Result<HighwayFrame, HighwayError> {
        let chunk = self.read_stream(&mut *data, self.chunk_size).await?;
        let mut md5_hasher = Md5::new();
        md5_hasher.update(&chunk);
        let chunk_md5 = md5_hasher.finalize().to_vec();
        let session = HighwaySession {
            ticket: self.ticket.clone(),
            uin: self.uin.clone(),
            cmd,
            command: String::from("PicUp.DataUp"),
            file_md5: md5.clone(),
            file_size: size,
            ext: ext_info.clone(),
        };
        let head = session.build_pic_up_head(offset, chunk.len() as u32, chunk_md5);
        Ok(HighwayFrame {
            head,
            body: chunk.into(),
        })
    }

    async fn parse_frame(&self, frame: HighwayFrame) -> Result<f64, HighwayError> {
        let mut buf = BytesMut::new();
        let mut codec = HighwayFrameCodec;
        codec.encode(frame, &mut buf).map_err(|e| {
            HighwayError::OtherError(format!("encode http frame failed: {}", e).into())
        })?;
        let mut codec = HighwayFrameCodec;
        let mut res_buf = BytesMut::from(buf.as_ref() as &[u8]);
        let res = match codec.decode(&mut res_buf)? {
            Some(frame) => frame,
            None => {
                return Err(HighwayError::OtherError(
                    "decode http frame: not enough data".into(),
                ));
            }
        };
        let res_head = RespDataHighwayHead::decode(res.head.as_ref()).unwrap();
        if res_head.error_code != 0 {
            return Err(HighwayError::UploadError(res_head.error_code));
        }
        let res_seg_head = res_head
            .msg_seg_head
            .ok_or(HighwayError::OtherError(Cow::from(
                "No seg head in response",
            )))?;
        let data_offset = res_seg_head
            .data_offset
            .ok_or(HighwayError::OtherError(Cow::from(
                "No data offset in response",
            )))?;
        let data_length = res_seg_head.data_length as u64;
        let data_size = res_seg_head.filesize;
        let percent = (data_offset + data_length) as f64 / data_size as f64 * 100.0;
        Ok(percent)
    }

    pub async fn upload(
        &self,
        cmd: u32,
        data: &mut AsyncPureStream,
        size: u32,
        md5: Bytes,
        ext_info: Bytes,
    ) -> Result<(), HighwayError> {
        match self
            .tcp_upload(cmd, data, size, md5.clone(), ext_info.clone())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("Tcp upload failed: {:?} fallback to http upload...", e);
                self.http_upload(cmd, data, size, md5, ext_info).await
            }
        }
    }

    async fn tcp_upload(
        &self,
        cmd: u32,
        data: &mut AsyncPureStream,
        size: u32,
        md5: Bytes,
        ext_info: Bytes,
    ) -> Result<(), HighwayError> {
        data.seek(io::SeekFrom::Start(0)).await?;
        let addr = self
            .addr
            .to_socket_addrs()?
            .next()
            .ok_or(HighwayError::OtherError(Cow::from("Invalid address")))?;
        let stream = tcp_connect_timeout(addr, Duration::from_secs(self.timeout as u64)).await?;
        let mut stream = Framed::new(stream, HighwayFrameCodec);
        for offset in (0..size).step_by(self.chunk_size) {
            let frame = self
                .build_frame(cmd, data, size, md5.clone(), ext_info.clone(), offset)
                .await?;
            stream.send(frame).await?;
            let res = loop {
                if let Some(resp) = stream.next().await {
                    break resp;
                }
            }?;
            let percent = self.parse_frame(res).await?;
            tracing::debug!("Highway TcpUpload Progress: {:.2}%", percent);
        }
        Ok(())
    }

    async fn http_upload(
        &self,
        cmd: u32,
        data: &mut AsyncPureStream,
        size: u32,
        md5: Bytes,
        ext_info: Bytes,
    ) -> Result<(), HighwayError> {
        data.seek(io::SeekFrom::Start(0)).await?;
        for offset in (0..size).step_by(self.chunk_size) {
            let frame = self
                .build_frame(cmd, data, size, md5.clone(), ext_info.clone(), offset)
                .await?;
            let mut codec = HighwayFrameCodec;
            let mut encoded_buf = BytesMut::new();
            codec.encode(frame, &mut encoded_buf).map_err(|e| {
                HighwayError::OtherError(format!("encode http frame failed: {}", e).into())
            })?;
            let post_frame = encoded_buf.freeze();
            let headers = [
                ("Connection", HeaderValue::from_static("keep-alive")),
                ("Accept-Encoding", HeaderValue::from_static("identity")),
                (
                    "User-Agent",
                    HeaderValue::from_static("Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.2)"),
                ),
                (
                    "Content-Length",
                    HeaderValue::from_str(&post_frame.len().to_string()).unwrap(),
                ),
            ]
            .into_iter()
            .map(|(k, v)| (k.parse::<HeaderName>().unwrap(), v))
            .collect::<HeaderMap>();
            let res = http::client()
                .post_binary_async(
                    format!(
                        "http://{}/cgi-bin/httpconn?htcmd=0x6FF0087&uin={}", // TODO: dynamic address
                        self.addr, self.uin
                    )
                    .as_str(),
                    &post_frame,
                    Some(headers),
                )
                .await
                .map_err(|e| {
                    HighwayError::OtherError(Cow::from(format!(
                        "Failed to upload via http: {:?}",
                        e
                    )))
                })?;
            let mut res_buf = BytesMut::from(res.as_ref() as &[u8]);
            let mut codec = HighwayFrameCodec;
            let res_frame = match codec.decode(&mut res_buf)? {
                Some(frame) => frame,
                None => {
                    return Err(HighwayError::OtherError(
                        "decode http frame: not enough data".into(),
                    ));
                }
            };
            let percent = self.parse_frame(res_frame).await?;
            tracing::debug!("Highway HttpUpload Progress: {:.2}%", percent);
        }
        Ok(())
    }
}
