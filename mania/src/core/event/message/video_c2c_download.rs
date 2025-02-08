use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    C2cUserInfo, ClientMeta, CommonHead, DownloadExt, DownloadReq, FileInfo, FileType, IndexNode,
    MultiMediaReqHead, Ntv2RichMediaReq, Ntv2RichMediaResp, SceneInfo, VideoDownloadExt,
};

/// legacy: VideoDownloadEvent
#[oidb_command(0x11e9, 200)]
#[derive(Debug, ServerEvent, Default)]
pub struct VideoC2CDownloadEvent {
    pub video_url: String,
    pub uuid: String,
    pub self_uid: String,
    pub file_name: String,
    pub file_md5: String,
    pub file_sha1: Option<String>,
    pub node: Option<IndexNode>,
    pub is_group: bool,
}

impl ClientEvent for VideoC2CDownloadEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let packet = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: match self.is_group {
                        true => 3,
                        false => 34,
                    },
                    command: 200,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 2,
                    business_type: 2,
                    scene_type: 1,
                    c2c: Some(C2cUserInfo {
                        account_type: 2,
                        target_uid: self.self_uid.to_owned(),
                    }),
                })),
                client: Some(ClientMeta { agent_type: 2 }),
            }),
            download: Some(DownloadReq {
                node: Some(self.node.to_owned().unwrap_or_else(|| (IndexNode {
                    info: Some(FileInfo {
                        file_size: 0,
                        file_hash: self.file_md5.to_owned(),
                        file_sha1: self.file_sha1.to_owned().unwrap_or_default(),
                        file_name: self.file_name.to_owned(),
                        r#type: Some(FileType {
                            r#type: 2,
                            pic_format: 0,
                            video_format: 0,
                            voice_format: 0,
                        }),
                        width: 0,
                        height: 0,
                        time: 0,
                        original: 0,
                    }),
                    file_uuid: self.uuid.to_owned(),
                    store_id: 0,
                    upload_time: 0,
                    ttl: 0,
                    sub_type: 0,
                }))),
                download: Some(dda!(DownloadExt {
                    video: Some(dda!(VideoDownloadExt {
                        busi_type: 0,
                        scene_type: 0,
                    }))
                })),
            }),
        });
        OidbPacket::new(0x11E9, 200, packet.encode_to_vec(), false, true).to_binary()
    }

    fn parse(packet: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        let packet = OidbPacket::parse_into::<Ntv2RichMediaResp>(packet)?;
        let download = packet.download.ok_or_else(|| {
            EventError::OtherError("Missing Ntv2RichMediaResp download response".to_string())
        })?;
        let info = download.info.as_ref().ok_or_else(|| {
            EventError::OtherError("Missing Ntv2RichMediaResp download info".to_string())
        })?;
        let url = format!(
            "https://{}{}{}",
            info.domain, info.url_path, download.r_key_param
        );
        Ok(Box::new(dda!(VideoC2CDownloadEvent { video_url: url })))
    }
}
