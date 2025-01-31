use prost::Message;
use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{ClientMeta, CommonHead, DownloadRKeyReq, MultiMediaReqHead, Ntv2RichMediaReq, SceneInfo};

#[ce_commend("OidbSvcTrpcTcp.0x9067_202")]
#[derive(Debug, ServerEvent)]
pub struct FetchRKeyEvent;

impl ClientEvent for FetchRKeyEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let request = Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 1,
                    command: 202,
                }),
                scene: Some(SceneInfo {
                    request_type: 2,
                    business_type: 1,
                    scene_type: 0,
                    ..Default::default()
                }),
                client: Some(ClientMeta {
                    agent_type: 2,
                    ..Default::default()
                }),
            }),
            download_r_key: Some(DownloadRKeyReq {
                types: vec![10, 20],
            }),
            ..Default::default()
        };
        let body = request.encode_to_vec();
        BinaryPacket::oidb(0x9067, 202, body, false, true)
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        Ok(Box::new(Self {}))
    }
}
