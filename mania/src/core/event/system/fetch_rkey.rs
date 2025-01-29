use crate::core::event::prelude::*;
use crate::core::protos::NTV2RichMediaReq::{
    ClientMeta, CommonHead, DownloadRKeyReq, MultiMediaReqHead, NTV2RichMediaReq, SceneInfo,
};
use protobuf::MessageField;

#[ce_commend("OidbSvcTrpcTcp.0x9067_202")]
#[derive(Debug, ServerEvent)]
pub struct FetchRKeyEvent;

impl ClientEvent for FetchRKeyEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let request = NTV2RichMediaReq {
            ReqHead: MessageField::from(Some(MultiMediaReqHead {
                Common: MessageField::from(Some(CommonHead {
                    RequestId: 1,
                    Command: 202,
                    ..Default::default()
                })),
                Scene: MessageField::from(Some(SceneInfo {
                    RequestType: 2,
                    BusinessType: 1,
                    SceneType: 0,
                    ..Default::default()
                })),
                Client: MessageField::from(Some(ClientMeta {
                    AgentType: 2,
                    ..Default::default()
                })),
                special_fields: Default::default(),
            })),
            DownloadRKey: MessageField::from(Some(DownloadRKeyReq {
                Types: vec![10, 20, 2],
                ..Default::default()
            })),
            ..Default::default()
        };
        let body = request.write_to_bytes().unwrap();
        BinaryPacket::oidb(0x9067, 202, body, false, true)
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        Ok(Box::new(Self {}))
    }
}
