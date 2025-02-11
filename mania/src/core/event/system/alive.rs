use crate::core::event::prelude::*;

#[command("Heartbeat.Alive")]
#[derive(Debug, ServerEvent)]
pub struct AliveEvent;

impl ClientEvent for AliveEvent {
    fn packet_type(&self) -> PacketType {
        PacketType::T13
    }

    fn build(&self, _: &Context) -> CEBuildResult {
        Ok(BinaryPacket(4u32.to_be_bytes().to_vec().into()))
    }

    fn parse(_: Bytes, _: &Context) -> CEParseResult {
        Ok(ClientResult::single(Box::new(Self {})))
    }
}
