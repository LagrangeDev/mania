use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    OidbSvcTrpcTcp0x6D6, OidbSvcTrpcTcp0x6D6Download, OidbSvcTrpcTcp0x6D6Response,
};

#[oidb_command(0x6d6, 2)]
#[derive(Debug, ServerEvent, Default)]
pub struct FileGroupDownloadEvent {
    pub group_uin: u32,
    pub file_url: String,
    pub file_id: String,
}

impl ClientEvent for FileGroupDownloadEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let packet = dda!(OidbSvcTrpcTcp0x6D6 {
            download: Some(OidbSvcTrpcTcp0x6D6Download {
                group_uin: self.group_uin,
                app_id: 7,
                bus_id: 102,
                file_id: self.file_id.to_owned(),
            }),
        });
        Ok(OidbPacket::new(0x6D6, 2, packet.encode_to_vec(), false, true).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let packet = OidbPacket::parse_into::<OidbSvcTrpcTcp0x6D6Response>(packet)?;
        let download = packet.download.ok_or_else(|| {
            EventError::OtherError("Missing OidbSvcTrpcTcp0x6D62response".to_string())
        })?;
        match download.ret_code {
            0 => {
                let url = format!(
                    "https://{}/ftn_handler/{}/?fname=",
                    download.download_dns,
                    hex::encode(download.download_url)
                );
                Ok(ClientResult::single(Box::new(dda!(Self { file_url: url }))))
            }
            _ => Err(EventError::OidbPacketInternalError(
                download.ret_code,
                download.client_wording,
            )),
        }
    }
}
