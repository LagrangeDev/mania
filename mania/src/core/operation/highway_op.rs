use crate::core::business::BusinessHandle;
use crate::core::event::downcast_major_event;
use crate::core::event::message::image_group_upload::{
    ImageGroupUploadArgs, ImageGroupUploadEvent,
};
use crate::core::event::system::fetch_highway_ticket::FetchHighwayTicketEvent;
use crate::core::highway::hw_client::HighwayClient;
use crate::core::highway::{AsyncPureStream, AsyncStream, oidb_ipv4s_to_highway_ipv4s};
use crate::core::protos::service::highway::{
    NtHighwayHash, NtHighwayNetwork, Ntv2RichMediaHighwayExt,
};
use crate::message::entity::image::ImageEntity;
use crate::utility::image_resolver::{ImageFormat, resolve_image_metadata};
use crate::utility::stream_helper::{mut_stream_ctx, stream_pipeline};
use crate::{ManiaError, ManiaResult, dda};
use bytes::Bytes;
use md5::Md5;
use prost::Message;
use sha1::{Digest, Sha1};
use std::borrow::Cow;
use std::sync::Arc;

impl BusinessHandle {
    async fn fetch_sig_session(self: &Arc<Self>) -> ManiaResult<Bytes> {
        let mut req = FetchHighwayTicketEvent::default();
        let req = self.send_event(&mut req).await?;
        let res: &FetchHighwayTicketEvent =
            downcast_major_event(&req).ok_or(ManiaError::InternalEventDowncastError)?;
        tracing::debug!("Fetched sig session: {:?}", res.sig_session);
        self.highway
            .sig_session
            .store(Arc::from(Some(res.sig_session.to_owned())));
        Ok(res.sig_session.to_owned())
    }

    async fn prepare_highway(self: &Arc<Self>) -> ManiaResult<()> {
        let _guard = self.highway.prepare_guard.lock().await;
        let sig = match self.highway.sig_session.load().as_ref() {
            Some(sig) => sig.clone(),
            None => self.fetch_sig_session().await?,
        };
        self.highway.client.store(Arc::new(HighwayClient::new(
            "htdata3.qq.com:80",
            60,
            sig,
            **self.context.key_store.uin.load(),
            self.context.config.highway_chuck_size,
        )));
        Ok(())
    }

    async fn resolve_image(
        self: &Arc<Self>,
        stream_locker: AsyncStream,
    ) -> ManiaResult<((ImageFormat, u32, u32), Bytes, Bytes)> {
        let (iv, sha1_bytes, md5_bytes) = mut_stream_ctx(&stream_locker, |s| {
            Box::pin(async move {
                let mut sha1_hasher = Sha1::new();
                let mut md5_hasher = Md5::new();
                stream_pipeline(s, |chunk| {
                    sha1_hasher.update(chunk);
                    md5_hasher.update(chunk);
                })
                .await?;
                let iv = resolve_image_metadata(s).await.map_err(|e| {
                    ManiaError::GenericError(Cow::from(format!("Resolve image error: {:?}", e)))
                })?;
                let sha1_bytes = Bytes::from(sha1_hasher.finalize().to_vec());
                let md5_bytes = Bytes::from(md5_hasher.finalize().to_vec());
                Ok::<((ImageFormat, u32, u32), Bytes, Bytes), ManiaError>((
                    iv, sha1_bytes, md5_bytes,
                ))
            })
        })
        .await?;
        Ok((iv, sha1_bytes, md5_bytes))
    }

    pub async fn upload_group_image(
        self: &Arc<Self>,
        group_uin: u32,
        image: &mut ImageEntity,
    ) -> ManiaResult<()> {
        self.prepare_highway().await?;
        let stream = match (&image.file_path, &image.image_stream, &mut image.size) {
            (Some(file_path), _, size) => {
                let file = tokio::fs::File::open(file_path).await?;
                *size = file.metadata().await?.len() as u32;
                Arc::new(tokio::sync::Mutex::new(Box::new(file) as AsyncPureStream))
            }
            (_, Some(stream), _) => stream.clone(),
            _ => {
                return Err(ManiaError::GenericError(Cow::from(
                    "No image stream or file path",
                )));
            }
        };

        let (iv, sha1, md5) = self.resolve_image(stream.clone()).await?;
        let mut req = dda!(ImageGroupUploadEvent {
            req: ImageGroupUploadArgs {
                group_uin,
                size: image.size,
                name: image.file_path.clone().unwrap_or_else(|| format!(
                    "{}.{}",
                    hex::encode(&sha1),
                    iv.0
                )),
                md5,
                sha1,
                pic_type: iv.0 as u32,
                sub_type: image.sub_type,
                summary: image.summary.clone().unwrap_or("[图片]".to_string()),
                width: iv.1,
                height: iv.2,
            },
        });
        let res = self.send_event(&mut req).await?;
        let res: &ImageGroupUploadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        if res.res.u_key.as_ref().is_some() {
            tracing::debug!(
                "uploadGroupImageReq get upload u_key: {}, need upload!",
                res.res.u_key.as_ref().unwrap()
            );
            let size = image.size;
            let chunk_size = self.context.config.highway_chuck_size;
            let msg_info_body = res.res.msg_info.msg_info_body.to_owned();
            let index_node = msg_info_body
                .first()
                .ok_or(ManiaError::GenericError(Cow::from(
                    "No index node in response",
                )))?
                .index
                .as_ref()
                .ok_or(ManiaError::GenericError(Cow::from("No index in response")))?;
            let info = index_node
                .info
                .as_ref()
                .ok_or(ManiaError::GenericError(Cow::from("No info in response")))?;
            let sha1 = hex::decode(&info.file_sha1).map_err(|e| {
                ManiaError::GenericError(Cow::from(format!("Hex decode error: {:?}", e)))
            })?;
            let md5 = hex::decode(&info.file_hash).map_err(|e| {
                ManiaError::GenericError(Cow::from(format!("Hex decode error: {:?}", e)))
            })?;
            let extend = Ntv2RichMediaHighwayExt {
                file_uuid: index_node.file_uuid.to_owned(),
                u_key: res.res.u_key.to_owned().unwrap(),
                network: Some(NtHighwayNetwork {
                    i_pv4s: oidb_ipv4s_to_highway_ipv4s(&res.res.ipv4s),
                }),
                msg_info_body: msg_info_body.to_owned(),
                block_size: chunk_size as u32,
                hash: Some({
                    NtHighwayHash {
                        file_sha1: vec![sha1],
                    }
                }),
            }
            .encode_to_vec();
            let client = self.highway.client.load();
            mut_stream_ctx(&stream, |s| {
                Box::pin(async move {
                    client
                        .upload(1004, s, size, Bytes::from(md5), Bytes::from(extend))
                        .await?;
                    Ok::<(), ManiaError>(())
                })
            })
            .await?;
            tracing::debug!("Successfully uploaded group image!");
        } else {
            tracing::debug!("No u_key in response, skip upload!");
        }
        image.msg_info = Some(res.res.msg_info.to_owned());
        image.custom_face = res.res.custom_face.to_owned();
        Ok(())
    }
}
