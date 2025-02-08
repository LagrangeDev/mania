use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    ClientMeta, CommonHead, DownloadRKeyReq, MultiMediaReqHead, Ntv2RichMediaReq, SceneInfo,
};

#[oidb_command(0x9067, 202)]
#[derive(Debug, ServerEvent)]
pub struct FetchRKeyEvent;

impl ClientEvent for FetchRKeyEvent {
    fn build(&self, _: &Context) -> Result<BinaryPacket, EventError> {
        let request = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 1,
                    command: 202,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 2,
                    business_type: 1,
                    scene_type: 0,
                })),
                client: Some(ClientMeta { agent_type: 2 }),
            }),
            download_r_key: Some(DownloadRKeyReq {
                types: vec![10, 20],
            }),
        });
        let body = request.encode_to_vec();
        Ok(OidbPacket::new(0x9067, 202, body, false, true).to_binary())
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        Ok(Box::new(Self {})) // TODO: parse
    }
}
