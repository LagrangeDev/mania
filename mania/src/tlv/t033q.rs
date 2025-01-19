use crate::tlv::prelude::*;
pub struct T033q {
    pub guid: Uuid,
}

impl TlvSer for T033q {
    fn from_context(ctx: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            guid: ctx.device.uuid,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x033, |p| p.bytes(self.guid.to_bytes()))
    }
}
