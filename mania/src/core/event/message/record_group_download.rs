use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    ClientMeta, CommonHead, DownloadExt, DownloadReq, IndexNode, MultiMediaReqHead, NtGroupInfo,
    Ntv2RichMediaReq, Ntv2RichMediaResp, SceneInfo, VideoDownloadExt,
};

#[commend("OidbSvcTrpcTcp.0x126e_200")]
#[derive(Debug, ServerEvent, Default)]
pub struct RecordGroupDownloadEvent {
    pub group_uin: u32,
    pub node: Option<IndexNode>,
    pub file_uuid: String,
    pub audio_url: String,
}

impl ClientEvent for RecordGroupDownloadEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let packet = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 4,
                    command: 200,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 1,
                    business_type: 3,
                    scene_type: 2,
                    group: Some(NtGroupInfo {
                        group_uin: self.group_uin,
                    }),
                })),
                client: Some(ClientMeta { agent_type: 2 }),
            }),
            download: Some(DownloadReq {
                node: Some(self.node.clone().unwrap_or(dda!(IndexNode {
                    file_uuid: self.file_uuid.to_owned(), // TODO: mut?
                }),)),
                download: Some(dda!(DownloadExt {
                    video: Some(dda!(VideoDownloadExt {
                        busi_type: 0,
                        scene_type: 0,
                    }))
                })),
            }),
        });
        OidbPacket::new(0x126E, 200, packet.encode_to_vec(), false, true).to_binary()
    }

    fn parse(packet: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        let packet = OidbPacket::parse_into::<Ntv2RichMediaResp>(packet)?;
        let download = packet.download.ok_or(EventError::OtherError(
            "Missing Ntv2RichMediaResp download response".to_string(),
        ))?;
        let info = download.info.as_ref().ok_or(EventError::OtherError(
            "Missing Ntv2RichMediaResp download info".to_string(),
        ))?;
        let url = format!(
            "https://{}{}{}",
            info.domain, info.url_path, download.r_key_param
        );
        Ok(Box::new(dda!(RecordGroupDownloadEvent { audio_url: url })))
    }
}
