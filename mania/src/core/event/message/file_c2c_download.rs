use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    OidbSvcTrpcTcp0xE371200, OidbSvcTrpcTcp0xE371200body, OidbSvcTrpcTcp0xE371200response,
};

#[oidb_command(0xe37, 1200)]
#[derive(Debug, ServerEvent, Default)]
pub struct FileC2CDownloadEvent {
    pub sender_uid: Option<String>,
    pub receiver_uid: Option<String>,
    pub file_uuid: Option<String>,
    pub file_hash: Option<String>,
    pub file_url: String,
}

impl ClientEvent for FileC2CDownloadEvent {
    fn build(&self, _: &Context) -> Result<BinaryPacket, EventError> {
        // TODO:
        // if (input.FileUuid == null || input.FileHash == null) throw new ArgumentNullException();
        // if (input.SenderUid == null || input.ReceiverUid == null) throw new ArgumentNullException();
        let packet = OidbSvcTrpcTcp0xE371200 {
            sub_command: 1200,
            field2: 1,
            body: Some(OidbSvcTrpcTcp0xE371200body {
                receiver_uid: self
                    .receiver_uid
                    .to_owned()
                    .ok_or_else(|| EventError::OtherError("Missing receiver_uid".to_string()))?,
                file_uuid: self
                    .file_uuid
                    .to_owned()
                    .ok_or_else(|| EventError::OtherError("Missing file_uuid".to_string()))?,
                r#type: 2,
                file_hash: self
                    .file_hash
                    .to_owned()
                    .ok_or_else(|| EventError::OtherError("Missing file_hash".to_string()))?,
                t2: 0,
            }),
            field101: 3,
            field102: 103,
            field200: 1,
            field99999: vec![0xC0, 0x85, 0x2C, 0x01],
        };
        Ok(OidbPacket::new(0xe37, 1200, packet.encode_to_vec(), false, false).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        let packet = OidbPacket::parse_into::<OidbSvcTrpcTcp0xE371200response>(packet)?;
        let body = packet.body.ok_or_else(|| {
            EventError::OtherError("Missing OidbSvcTrpcTcp0xE371200responseBody".to_string())
        })?;
        let result = body.result.ok_or_else(|| {
            EventError::OtherError("Missing OidbSvcTrpcTcp0xE371200result".to_string())
        })?;
        let file_url = format!(
            "https://{}:{}{}&isthumb=0",
            result.sso_url, result.sso_port, result.url
        );
        Ok(Box::new(dda!(Self { file_url })))
    }
}
