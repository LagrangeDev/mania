use crate::core::event::prelude::*;
use crate::core::protos::message::{
    LongMsgResult, LongMsgSettings, LongMsgUid, RecvLongMsgInfo, RecvLongMsgReq, RecvLongMsgResp,
};
use crate::message::chain::MessageChain;
use crate::message::packer::MessagePacker;
use crate::utility::compress::gzip;
use mania_macros::{ServerEvent, command};

#[command("trpc.group.long_msg_interface.MsgService.SsoRecvLongMsg")]
#[derive(Debug, ServerEvent, Default)]
pub struct MultiMsgDownloadEvent {
    pub uid: Option<String>,
    pub res_id: Option<String>,
    pub chains: Option<Vec<MessageChain>>,
}

impl ClientEvent for MultiMsgDownloadEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let packet = RecvLongMsgReq {
            info: Some(RecvLongMsgInfo {
                uid: Some(LongMsgUid {
                    uid: self.uid.clone(),
                }),
                res_id: self.res_id.clone(),
                acquire: true,
            }),
            settings: Some(LongMsgSettings {
                field1: 2,
                field2: 0,
                field3: 0,
                field4: 0,
            }),
        };
        Ok(BinaryPacket(packet.encode_to_vec().into()))
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let packet = RecvLongMsgResp::decode(packet)?;
        let inflate = packet
            .result
            .ok_or_else(|| EventError::OtherError("Missing RecvLongMsgInfo".to_string()))?
            .payload;
        let inflate = gzip::decompress(&inflate).ok_or_else(|| {
            EventError::OtherError("Failed to decompress long message".to_string())
        })?;
        let result = LongMsgResult::decode(Bytes::from(inflate))?;
        let main = result
            .action
            .into_iter()
            .find(|a| a.action_command == "MultiMsg")
            .ok_or_else(|| EventError::OtherError("Failed to find MultiMsg command".to_string()))?;
        let chains = main
            .action_data
            .ok_or_else(|| EventError::OtherError("Failed to find action_data".to_string()))?
            .msg_body
            .into_iter()
            .map(MessagePacker::parse_fake_chain)
            .collect::<Result<Vec<MessageChain>, String>>()
            .map_err(EventError::OtherError)?;
        Ok(ClientResult::single(Box::new(dda!(
            MultiMsgDownloadEvent {
                chains: Some(chains),
            }
        ))))
    }
}
