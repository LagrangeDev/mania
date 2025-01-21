use crate::tlv::prelude::*;

pub struct T145 {
    pub device_guid: Uuid,
}

impl TlvSer for T145 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            device_guid: ctx.device.uuid,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x145, |p| p.bytes(self.device_guid.to_bytes()))
    }
}
