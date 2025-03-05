use std::{io::Cursor, sync::Arc};

use image::ImageResult;

use super::prelude::*;
use crate::{
    core::{
        highway::AsyncPureStream,
        protos::service::oidb::{IndexNode, MsgInfo},
    },
    utility::image_gen::gen_thumbnail,
};

#[pack_content(false)]
#[derive(Default)]
pub struct VideoEntity {
    pub file_name: String,
    pub video_hash: String,
    pub height: i32,
    pub width: i32,
    pub video_size: i32,
    pub video_length: i32,
    pub video_thumb_size: i32,
    pub video_thumb_height: i32,
    pub video_thumb_width: i32,
    pub video_url: String,
    pub video_path: Option<String>,
    pub video_stream: Option<AsyncStream>,
    pub video_thumb_path: Option<String>,
    pub video_thumb_stream: Option<AsyncStream>,
    pub(crate) node: Option<IndexNode>, // for download, 2025/02/08
    pub(crate) video_uuid: Option<String>,
    pub(crate) msg_info: Option<MsgInfo>,
    pub(crate) compat: Option<VideoFile>,
}

impl VideoEntity {
    pub(crate) async fn resolve_stream(
        &mut self,
    ) -> Result<(Option<AsyncStream>, Option<AsyncStream>), String> {
        let load_stream = |path: String| async move {
            let file = tokio::fs::File::open(path).await.ok()?;
            let metadata = file.metadata().await.ok()?;
            let size = metadata.len() as i32;
            let stream = Arc::new(tokio::sync::Mutex::new(Box::new(file) as AsyncPureStream));
            Some((stream, size))
        };

        let video_stream = if let Some(video_path) = self.video_path.as_ref() {
            if let Some((stream, size)) = load_stream(video_path.clone()).await {
                self.video_size = size;
                Some(stream)
            } else {
                None
            }
        } else {
            self.video_stream.clone()
        };

        let video_thumb_stream = if let Some(thumb_path) = self.video_thumb_path.as_ref() {
            if let Some((stream, size)) = load_stream(thumb_path.clone()).await {
                self.video_thumb_size = size;
                Some(stream)
            } else {
                None
            }
        } else {
            match self.video_thumb_stream.as_ref() {
                Some(stream) => Some(stream.clone()),
                None => {
                    let res =
                        tokio::task::spawn_blocking(move || -> ImageResult<Cursor<Vec<u8>>> {
                            let mut thumb = Cursor::new(Vec::new());
                            gen_thumbnail(&mut thumb)?;
                            Ok(thumb)
                        })
                        .await
                        .map_err(|e| e.to_string());
                    if let Ok(Ok(thumb)) = res {
                        let size = thumb.get_ref().len() as i32;
                        let stream =
                            Arc::new(tokio::sync::Mutex::new(Box::new(thumb) as AsyncPureStream));
                        self.video_thumb_size = size;
                        Some(stream)
                    } else {
                        return Err(res.unwrap_err());
                    }
                }
            }
        };
        Ok((video_stream, video_thumb_stream))
    }
}

impl Debug for VideoEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[Video]: {}x{}: {} {}",
            self.width, self.height, self.video_size, self.video_url
        )
    }
}

impl Display for VideoEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[视频]")
    }
}

impl MessageEntity for VideoEntity {
    fn pack_element(&self, _: &Context) -> Vec<Elem> {
        let common = self.msg_info.as_ref().map_or_else(
            || MsgInfo::default().encode_to_vec(),
            |msg_info| msg_info.encode_to_vec(),
        );
        vec![dda!(Elem {
            common_elem: Some(CommonElem {
                service_type: 48,
                pb_elem: common,
                business_type: 21,
            }),
        })]
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        elem.video_file.as_ref().map(|video_file| {
            dda!(Self {
                video_hash: video_file.file_md5.hex(),
                height: video_file.file_height,
                width: video_file.file_width,
                video_size: video_file.file_size,
                video_uuid: Some(video_file.file_uuid.to_owned()),
            })
        });
        match elem.common_elem.as_ref() {
            Some(common) => {
                match (
                    common.service_type,
                    common.pb_elem.as_ref(),
                    common.business_type,
                ) {
                    (48, pb, _) => {
                        let msg_info = MsgInfo::decode(pb).ok()?;
                        let msg_info_body = msg_info.msg_info_body.first();
                        let node = msg_info_body?.index.to_owned()?;
                        let info = node.info.as_ref()?;
                        Some(dda!(Self {
                            file_name: info.file_name.clone(),
                            height: info.height as i32,
                            width: info.width as i32,
                            video_size: info.file_size as i32,
                            video_length: info.time as i32,
                            node: Some(node),
                        }))
                    }
                    _ => None,
                }
            }
            None => None,
        }
    }
}
