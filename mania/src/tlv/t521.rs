use crate::tlv::prelude::*;

pub struct T521 {
    pub product_type: u32,
}

impl TlvSer for T521 {
    fn from_context(_: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self { product_type: 0x13 })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x521, |p| {
            p.u32(self.product_type).string_with_length("basicim")
        })
    }
}
