use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    ClientMeta, CommonHead, DownloadExt, DownloadReq, FileInfo, FileType, IndexNode,
    MultiMediaReqHead, NtGroupInfo, Ntv2RichMediaReq, Ntv2RichMediaResp, SceneInfo,
    VideoDownloadExt,
};

#[oidb_command(0x11ea, 200)]
#[derive(Debug, ServerEvent, Default)]
pub struct VideoGroupDownloadEvent {
    pub video_url: String,
    pub uuid: String,
    pub self_uid: String,
    pub file_name: String,
    pub file_md5: String,
    pub file_sha1: Option<String>,
    pub node: Option<IndexNode>,
    pub group_uin: u32,
}

impl ClientEvent for VideoGroupDownloadEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let packet = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 1,
                    command: 200,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 2,
                    business_type: 2,
                    scene_type: 2,
                    group: Some(NtGroupInfo {
                        group_uin: self.group_uin,
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
                        scene_type: 0, // FIXME: in VideoDownloadExt 1: 0, 3: 0, 5: 0, 6: (maybe video thumb meta)
                    }))
                })),
            }),
        });
        Ok(OidbPacket::new(0x11EA, 200, packet.encode_to_vec(), false, true).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
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
        Ok(ClientResult::single(Box::new(dda!(
            VideoGroupDownloadEvent { video_url: url }
        ))))
    }
}
