use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    C2cUserInfo, ClientMeta, CommonHead, DownloadExt, DownloadReq, IndexNode, MultiMediaReqHead,
    Ntv2RichMediaReq, Ntv2RichMediaResp, SceneInfo, VideoDownloadExt,
};

#[commend("OidbSvcTrpcTcp.0x126d_200")]
#[derive(Debug, ServerEvent, Default)]
pub struct RecordC2CDownloadEvent {
    pub self_uid: String,
    pub node: Option<IndexNode>,
    pub file_uuid: String,
    pub audio_url: String,
}

impl ClientEvent for RecordC2CDownloadEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let packet = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 1,
                    command: 200,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 1,
                    business_type: 3,
                    scene_type: 1,
                    c2c: Some(C2cUserInfo {
                        account_type: 2,
                        target_uid: self.self_uid.to_owned(),
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
        OidbPacket::new(0x126D, 200, packet.encode_to_vec(), false, true).to_binary()
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
        Ok(Box::new(dda!(RecordC2CDownloadEvent { audio_url: url })))
    }
}
