use crate::core::tlv::prelude::*;

pub struct T166 {
    pub image_type: u8,
}

impl TlvSer for T166 {
    fn from_context(_: &Context) -> Box<dyn TlvSer> {
        Box::new(Self { image_type: 5 })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x166, |p| p.u8(self.image_type))
    }
}
