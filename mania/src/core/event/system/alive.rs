use crate::core::event::prelude::*;

#[ce_commend("Heartbeat.Alive")]
#[derive(Debug, ServerEvent)]
pub struct AliveEvent {
    pub test: u32,
}

impl ClientEvent for AliveEvent {
    fn packet_type(&self) -> PacketType {
        PacketType::T13
    }

    fn build(&self, _: &Context) -> BinaryPacket {
        BinaryPacket(4u32.to_be_bytes().to_vec().into())
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        Ok(Box::new(Self { test: 0 }))
    }
}
