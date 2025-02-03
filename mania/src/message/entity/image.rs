use super::prelude::*;
use crate::core::protos::service::oidb::MsgInfo;

#[derive(Default)]
pub struct ImageEntity {
    pub height: u32,
    pub width: u32,
    pub file_path: String,
    pub md5: Bytes,
    pub size: u32,
    pub url: String,
    // TODO: lazy stream
    pub(crate) msg_info: Option<MsgInfo>,
    pub(crate) not_online_image: NotOnlineImage,
    pub(crate) custom_face: CustomFace,
    pub summary: Option<String>,
    pub sub_type: u32,
    pub is_group: bool,
}

impl ImageEntity {
    fn to_preview_text(&self) -> String {
        match &self.summary {
            Some(summary) if !summary.is_empty() => summary.clone(),
            _ => match self.sub_type {
                1 => "[动画表情]".to_string(),
                _ => "[图片]".to_string(),
            },
        }
    }
}

impl Debug for ImageEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[Image: {}x{}] {} {} {} {}",
            self.width,
            self.height,
            self.to_preview_text(),
            self.file_path,
            self.size,
            self.url
        )
    }
}

impl Display for ImageEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_preview_text())
    }
}

impl MessageEntity for ImageEntity {
    fn pack_element(&self) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        if let Some(common) = &elem.common_elem {
            if common.service_type == 48
                && (common.business_type == 10 || common.business_type == 20)
            {
                let extra: MsgInfo = MsgInfo::decode(&*common.pb_elem).unwrap();
                let ext_biz_info = extra.ext_biz_info.as_ref().unwrap();
                let msg_info_body = &extra.msg_info_body[0];
                let index = msg_info_body.index.as_ref().unwrap();

                return Some(dda!(ImageEntity {
                    height: index.info.as_ref().unwrap().height,
                    width: index.info.as_ref().unwrap().width,
                    file_path: index.info.as_ref().unwrap().file_name.clone(),
                    md5: Bytes::from(hex::decode(&index.info.as_ref().unwrap().file_hash).unwrap()),
                    size: index.info.as_ref().unwrap().file_size,
                    msg_info: Some(extra.clone()),
                    sub_type: ext_biz_info.pic.as_ref().unwrap().biz_type,
                    is_group: ext_biz_info.pic.as_ref().unwrap().ext_data.is_some(),
                    summary: if ext_biz_info.pic.as_ref().unwrap().text_summary.is_empty() {
                        Some("[图片]".to_string())
                    } else {
                        Some(ext_biz_info.pic.as_ref().unwrap().text_summary.clone())
                    },
                }));
            }
        }

        if let Some(image) = &elem.not_online_image {
            let url = if image.orig_url.contains("&fileid=") {
                format!("https://multimedia.nt.qq.com.cn{}", image.orig_url)
            } else {
                format!("https://gchat.qpic.cn{}", image.orig_url)
            };
            let pb_res = if let Some(ref pb) = image.pb_res {
                pb
            } else {
                &not_online_image::PbReserve::default()
            };
            return Some(dda!(ImageEntity {
                height: image.pic_height,
                width: image.pic_width,
                file_path: image.file_path.clone(),
                md5: Bytes::from(image.pic_md5.clone()),
                size: image.file_len,
                url,
                not_online_image: image.clone(),
                sub_type: pb_res.sub_type as u32,
                is_group: false,
                summary: Some(pb_res.summary.clone()),
            }));
        }

        if let Some(face) = &elem.custom_face {
            let url = if face.orig_url.contains("&fileid=") {
                format!("https://multimedia.nt.qq.com.cn{}", face.orig_url)
            } else {
                format!("https://gchat.qpic.cn{}", face.orig_url)
            };
            let pb_res = if let Some(ref pb) = face.pb_res {
                pb
            } else {
                &custom_face::PbReserve::default()
            };
            return Some(dda!(ImageEntity {
                height: face.height as u32,
                width: face.width as u32,
                file_path: face.file_path.clone(),
                md5: Bytes::from(face.md5.clone()),
                size: face.size,
                url,
                custom_face: face.clone(),
                sub_type: pb_res.sub_type as u32,
                is_group: true,
                summary: Some(pb_res.summary.clone()),
            }));
        }

        None
    }
}
