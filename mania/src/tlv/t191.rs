use crate::tlv::prelude::*;

pub struct T191 {
    pub can_web_verify: u8,
}

impl TlvSer for T191 {
    fn from_context(_: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self { can_web_verify: 0 })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x191, |p| p.u8(self.can_web_verify))
    }
}
