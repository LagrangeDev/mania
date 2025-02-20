use crate::core::business::BusinessHandle;
use crate::core::crypto::stream_sha1::StreamSha1;
use crate::core::event::downcast_major_event;
use crate::core::event::message::image_c2c_upload::{ImageC2CUploadArgs, ImageC2CUploadEvent};
use crate::core::event::message::image_group_upload::{
    ImageGroupUploadArgs, ImageGroupUploadEvent,
};
use crate::core::event::message::video_c2c_upload::{VideoC2CUploadArgs, VideoC2CUploadEvent};
use crate::core::event::message::video_group_upload::{
    VideoGroupUploadArgs, VideoGroupUploadEvent,
};
use crate::core::event::system::fetch_highway_ticket::FetchHighwayTicketEvent;
use crate::core::highway::hw_client::HighwayClient;
use crate::core::highway::{AsyncStream, HighwayError, oidb_ipv4s_to_highway_ipv4s};
use crate::core::protos::service::highway::{
    NtHighwayHash, NtHighwayNetwork, Ntv2RichMediaHighwayExt,
};
use crate::message::entity::image::ImageEntity;
use crate::message::entity::video::VideoEntity;
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
            "htdata3.qq.com:80", // TODO: Configurable & dynamic
            60,
            sig,
            **self.context.key_store.uin.load(),
            self.context.config.highway_chuck_size,
        )));
        Ok(())
    }

    async fn resolve_image(
        self: &Arc<Self>,
        stream_ctx: AsyncStream,
    ) -> ManiaResult<((ImageFormat, u32, u32), Bytes, Bytes)> {
        let (iv, sha1_bytes, md5_bytes) = mut_stream_ctx(&stream_ctx, |s| {
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
        let stream = image
            .resolve_stream()
            .await
            .ok_or(ManiaError::GenericError(Cow::from("No image stream found")))?;

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
            let sha1 = hex::decode(&info.file_sha1).map_err(HighwayError::HexError)?;
            let md5 = hex::decode(&info.file_hash).map_err(HighwayError::HexError)?;
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
            tracing::debug!("No u_key in upload_group_image response, skip upload!");
        }
        image.msg_info = Some(res.res.msg_info.to_owned());
        image.custom_face = res.res.custom_face.to_owned();
        Ok(())
    }

    pub async fn upload_c2c_image(
        self: &Arc<Self>,
        target_uid: &str,
        image: &mut ImageEntity,
    ) -> ManiaResult<()> {
        self.prepare_highway().await?;
        let stream = image
            .resolve_stream()
            .await
            .ok_or(ManiaError::GenericError(Cow::from("No image stream found")))?;
        let (iv, sha1, md5) = self.resolve_image(stream.clone()).await?;
        let mut req = dda!(ImageC2CUploadEvent {
            req: ImageC2CUploadArgs {
                uid: target_uid.to_string(),
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
        let res: &ImageC2CUploadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        if res.res.u_key.as_ref().is_some() {
            tracing::debug!(
                "uploadC2CImageReq get upload u_key: {}, need upload!",
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
            let sha1 = hex::decode(&info.file_sha1).map_err(HighwayError::HexError)?;
            let md5 = hex::decode(&info.file_hash).map_err(HighwayError::HexError)?;
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
                        .upload(1003, s, size, Bytes::from(md5), Bytes::from(extend))
                        .await?;
                    Ok::<(), ManiaError>(())
                })
            })
            .await?;
            tracing::debug!("Successfully uploaded c2c image!");
        } else {
            tracing::debug!("No u_key in upload_c2c_image response, skip upload!");
        }
        image.msg_info = Some(res.res.msg_info.to_owned());
        image.not_online_image = res.res.not_online_image.to_owned();
        Ok(())
    }

    async fn resolve_video(
        self: &Arc<Self>,
        video_stream_ctx: AsyncStream,
        video_thumb_stream_ctx: AsyncStream,
    ) -> ManiaResult<(Bytes, Bytes, Vec<Vec<u8>>, Bytes, Bytes)> {
        let (file_md5, file_sha1, file_stream_sha1) = mut_stream_ctx(&video_stream_ctx, |s| {
            Box::pin(async move {
                let mut md5_hasher = Md5::new();
                let mut sha1_hasher = Sha1::new();
                let mut stream_sha1_hasher = StreamSha1::new();
                stream_pipeline(s, |chunk| {
                    md5_hasher.update(chunk);
                    sha1_hasher.update(chunk);
                    stream_sha1_hasher.update(chunk);
                })
                .await?;
                let md5 = Bytes::from(md5_hasher.finalize().to_vec());
                let sha1 = Bytes::from(sha1_hasher.finalize().to_vec());
                let stream_sha1 = stream_sha1_hasher.finalize();
                let stream_sha1 = stream_sha1.into_iter().map(|arr| arr.to_vec()).collect();
                Ok::<(Bytes, Bytes, Vec<Vec<u8>>), ManiaError>((md5, sha1, stream_sha1))
            })
        })
        .await?;
        let (thumb_md5, thumb_sha1) = mut_stream_ctx(&video_thumb_stream_ctx, |s| {
            Box::pin(async move {
                let mut md5_hasher = Md5::new();
                let mut sha1_hasher = Sha1::new();
                stream_pipeline(s, |chunk| {
                    md5_hasher.update(chunk);
                    sha1_hasher.update(chunk);
                })
                .await?;
                let md5 = Bytes::from(md5_hasher.finalize().to_vec());
                let sha1 = Bytes::from(sha1_hasher.finalize().to_vec());
                Ok::<(Bytes, Bytes), ManiaError>((md5, sha1))
            })
        })
        .await?;
        Ok((file_md5, file_sha1, file_stream_sha1, thumb_md5, thumb_sha1))
    }

    pub async fn upload_group_video(
        self: &Arc<Self>,
        group_uin: u32,
        video: &mut VideoEntity,
    ) -> ManiaResult<()> {
        self.prepare_highway().await?;
        let (vs, is) = video.resolve_stream().await;
        let vs = vs.ok_or(ManiaError::GenericError(Cow::from("No video stream found")))?;
        let is = is.ok_or(ManiaError::GenericError(Cow::from("No image stream found")))?;
        let (file_md5, file_sha1, file_stream_sha1, thumb_md5, thumb_sha1) =
            self.resolve_video(vs.clone(), is.clone()).await?;
        let mut req = dda!(VideoGroupUploadEvent {
            req: VideoGroupUploadArgs {
                group_uin,
                video_size: video.video_size as u32,
                video_name: video
                    .video_path
                    .clone()
                    .unwrap_or_else(|| { format!("{}.mp4", hex::encode(&file_sha1)) }),
                video_md5: file_md5,
                video_sha1: file_sha1,
                thumb_size: video.video_thumb_size as u32,
                thumb_name: video
                    .video_thumb_path
                    .clone()
                    .unwrap_or_else(|| { format!("{}.jpg", hex::encode(&thumb_sha1)) }),
                thumb_md5,
                thumb_sha1,
                thumb_width: video.video_thumb_width as u32,
                thumb_height: video.video_thumb_height as u32,
                summary: "[视频]".to_string(),
            }
        });
        let res = self.send_event(&mut req).await?;
        let res: &VideoGroupUploadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        let chunk_size = self.context.config.highway_chuck_size;
        if res.res.u_key.as_ref().is_some() {
            tracing::debug!(
                "uploadGroupVideoReq (Video) get upload u_key: {}, need upload!",
                res.res.u_key.as_ref().unwrap()
            );
            let size = video.video_size as u32;
            let msg_info_body = res.res.msg_info.msg_info_body.clone();
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
            let md5 = hex::decode(&info.file_hash).map_err(HighwayError::HexError)?;
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
                        file_sha1: file_stream_sha1,
                    }
                }),
            }
            .encode_to_vec();
            let client = self.highway.client.load();
            mut_stream_ctx(&vs, |s| {
                Box::pin(async move {
                    client
                        .upload(1005, s, size, Bytes::from(md5), Bytes::from(extend))
                        .await?;
                    Ok::<(), ManiaError>(())
                })
            })
            .await?;
        } else {
            tracing::debug!("No u_key in upload_group_video (Video) response, skip upload!");
        }
        if let Some(sub_file) = res.res.sub_file_info.first()
            && !sub_file.u_key.is_empty()
        {
            tracing::debug!(
                "uploadGroupVideoReq (Thumb) get upload u_key: {}, need upload!",
                sub_file.u_key
            );
            let msg_info_body = res.res.msg_info.msg_info_body.to_owned();
            let index = res
                .res
                .msg_info
                .msg_info_body
                .get(1)
                .ok_or(ManiaError::GenericError(Cow::from(
                    "No index node in response",
                )))?
                .index
                .as_ref()
                .ok_or(ManiaError::GenericError(Cow::from("No index in response")))?;
            let info = index
                .info
                .as_ref()
                .ok_or(ManiaError::GenericError(Cow::from("No info in response")))?;
            let md5 = hex::decode(&info.file_hash).map_err(HighwayError::HexError)?;
            let sha1 = hex::decode(&info.file_sha1).map_err(HighwayError::HexError)?;
            let size = video.video_thumb_size as u32;
            let extend = Ntv2RichMediaHighwayExt {
                file_uuid: index.file_uuid.to_owned(),
                u_key: sub_file.u_key.to_owned(),
                network: Some(NtHighwayNetwork {
                    i_pv4s: oidb_ipv4s_to_highway_ipv4s(&res.res.ipv4s),
                }),
                msg_info_body,
                block_size: chunk_size as u32,
                hash: Some({
                    NtHighwayHash {
                        file_sha1: vec![sha1],
                    }
                }),
            }
            .encode_to_vec();
            let client = self.highway.client.load();
            mut_stream_ctx(&is, |s| {
                Box::pin(async move {
                    client
                        .upload(1006, s, size, Bytes::from(md5), Bytes::from(extend))
                        .await?;
                    Ok::<(), ManiaError>(())
                })
            })
            .await?;
        } else {
            tracing::debug!("No u_key in upload_group_video (Thumb) response, skip upload!");
        }
        video.msg_info = Some(res.res.msg_info.to_owned());
        video.compat = Some(res.res.video_file.to_owned());
        Ok(())
    }

    pub async fn upload_c2c_video(
        self: &Arc<Self>,
        target_uid: &str,
        video: &mut VideoEntity,
    ) -> ManiaResult<()> {
        self.prepare_highway().await?;
        let (vs, is) = video.resolve_stream().await;
        let vs = vs.ok_or(ManiaError::GenericError(Cow::from("No video stream found")))?;
        let is = is.ok_or(ManiaError::GenericError(Cow::from("No image stream found")))?;
        let (file_md5, file_sha1, file_stream_sha1, thumb_md5, thumb_sha1) =
            self.resolve_video(vs.clone(), is.clone()).await?;
        let mut req = dda!(VideoC2CUploadEvent {
            req: VideoC2CUploadArgs {
                uid: target_uid.to_string(),
                video_size: video.video_size as u32,
                video_name: video
                    .video_path
                    .clone()
                    .unwrap_or_else(|| { format!("{}.mp4", hex::encode(&file_sha1)) }),
                video_md5: file_md5,
                video_sha1: file_sha1,
                thumb_size: video.video_thumb_size as u32,
                thumb_name: video
                    .video_thumb_path
                    .clone()
                    .unwrap_or_else(|| { format!("{}.jpg", hex::encode(&thumb_sha1)) }),
                thumb_md5,
                thumb_sha1,
                thumb_width: video.video_thumb_width as u32,
                thumb_height: video.video_thumb_height as u32,
                summary: "[视频]".to_string(),
            }
        });
        let res = self.send_event(&mut req).await?;
        let res: &VideoC2CUploadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        let chunk_size = self.context.config.highway_chuck_size;
        if res.res.u_key.as_ref().is_some() {
            tracing::debug!(
                "uploadC2CVideoReq (Video) get upload u_key: {}, need upload!",
                res.res.u_key.as_ref().unwrap()
            );
            let size = video.video_size as u32;
            let msg_info_body = res.res.msg_info.msg_info_body.clone();
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
            let md5 = hex::decode(&info.file_hash).map_err(HighwayError::HexError)?;
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
                        file_sha1: file_stream_sha1,
                    }
                }),
            }
            .encode_to_vec();
            let client = self.highway.client.load();
            mut_stream_ctx(&vs, |s| {
                Box::pin(async move {
                    client
                        .upload(1001, s, size, Bytes::from(md5), Bytes::from(extend))
                        .await?;
                    Ok::<(), ManiaError>(())
                })
            })
            .await?;
        } else {
            tracing::debug!("No u_key in upload_c2c_video (Video) response, skip upload!");
        }
        if let Some(sub_file) = res.res.sub_file_info.first()
            && !sub_file.u_key.is_empty()
        {
            tracing::debug!(
                "uploadC2CVideoReq (Thumb) get upload u_key: {}, need upload!",
                sub_file.u_key
            );
            let msg_info_body = res.res.msg_info.msg_info_body.to_owned();
            let index = res
                .res
                .msg_info
                .msg_info_body
                .get(1)
                .ok_or(ManiaError::GenericError(Cow::from(
                    "No index node in response",
                )))?
                .index
                .as_ref()
                .ok_or(ManiaError::GenericError(Cow::from("No index in response")))?;
            let info = index
                .info
                .as_ref()
                .ok_or(ManiaError::GenericError(Cow::from("No info in response")))?;
            let md5 = hex::decode(&info.file_hash).map_err(HighwayError::HexError)?;
            let sha1 = hex::decode(&info.file_sha1).map_err(HighwayError::HexError)?;
            let size = video.video_thumb_size as u32;
            let extend = Ntv2RichMediaHighwayExt {
                file_uuid: index.file_uuid.to_owned(),
                u_key: sub_file.u_key.to_owned(),
                network: Some(NtHighwayNetwork {
                    i_pv4s: oidb_ipv4s_to_highway_ipv4s(&res.res.ipv4s),
                }),
                msg_info_body,
                block_size: chunk_size as u32,
                hash: Some({
                    NtHighwayHash {
                        file_sha1: vec![sha1],
                    }
                }),
            }
            .encode_to_vec();
            let client = self.highway.client.load();
            mut_stream_ctx(&is, |s| {
                Box::pin(async move {
                    client
                        .upload(1002, s, size, Bytes::from(md5), Bytes::from(extend))
                        .await?;
                    Ok::<(), ManiaError>(())
                })
            })
            .await?;
        } else {
            tracing::debug!("No u_key in upload_c2c_video (Thumb) response, skip upload!");
        }
        video.msg_info = Some(res.res.msg_info.to_owned());
        video.compat = Some(res.res.video_file.to_owned());
        Ok(())
    }
}
