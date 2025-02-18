use crate::core::event::prelude::*;
use crate::core::protos::action::SendMessageResponse;
use crate::message::chain::MessageChain;
use crate::message::packer::MessagePacker;

#[command("MessageSvc.PbSendMsg")]
#[derive(Debug, ServerEvent, Default)]
pub struct SendMessageEvent {
    pub chain: MessageChain,
    pub result: MessageResult,
}

#[derive(Debug, Default)]
pub struct MessageResult {
    pub message_id: u64,
    pub sequence: Option<u32>,
    pub result: u32,
    pub timestamp: u32,
    pub client_sequence: u32,
}

impl ClientEvent for SendMessageEvent {
    fn build(&self, ctx: &Context) -> CEBuildResult {
        let packet = MessagePacker::build(&self.chain, ctx);
        Ok(BinaryPacket(packet.encode_to_vec().into()))
    }

    fn parse(bytes: Bytes, _: &Context) -> CEParseResult {
        let res = SendMessageResponse::decode(bytes)?;
        let result = dda!(MessageResult {
            result: res.result as u32,
            sequence: Some(res.group_sequence.unwrap_or(res.private_sequence)),
            timestamp: res.timestamp1,
        });
        Ok(ClientResult::single(Box::new(dda!(SendMessageEvent {
            result,
        }))))
    }
}
