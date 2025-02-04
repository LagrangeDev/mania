use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    ClientMeta, CommonHead, DownloadExt, DownloadReq, IndexNode, MultiMediaReqHead, NtGroupInfo,
    Ntv2RichMediaReq, Ntv2RichMediaResp, SceneInfo, VideoDownloadExt,
};

#[commend("OidbSvcTrpcTcp.0x11c4_200")]
#[derive(Debug, ServerEvent, Default)]
pub struct ImageGroupDownloadEvent {
    pub group_uin: u32,
    pub index_node: IndexNode,
    pub image_url: String,
}

impl ClientEvent for ImageGroupDownloadEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let request = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 1,
                    command: 200,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 2,
                    business_type: 1,
                    scene_type: 2,
                    group: Some(NtGroupInfo {
                        group_uin: self.group_uin,
                    }),
                })),
                client: Some(ClientMeta { agent_type: 2 }),
            }),
            download: Some(DownloadReq {
                node: Some(self.index_node.clone()),
                download: Some(dda!(DownloadExt {
                    video: Some(dda!(VideoDownloadExt {
                        busi_type: 0,
                        scene_type: 0
                    })),
                })),
            })
        });
        let body = request.encode_to_vec();
        OidbPacket::new(0x11c4, 200, body, false, true).to_binary()
    }

    fn parse(packet: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        let packet = OidbPacket::parse_into::<Ntv2RichMediaResp>(packet)
            .expect("Failed to parse OidbPacket");
        let body = packet
            .download
            .ok_or(EventError::OtherError("Missing DownloadResp".to_string()))?;
        let info = body
            .info
            .ok_or(EventError::OtherError("Missing DownloadInfo".to_string()))?;
        let url = format!(
            "https://{}{}{}",
            info.domain, info.url_path, body.r_key_param
        );
        Ok(Box::new(dda!(Self { image_url: url })))
    }
}
