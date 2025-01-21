use crate::tlv::prelude::*;

pub struct T124 {}

impl TlvSer for T124 {
    fn from_context(_: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {})
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x124, |p| p.bytes(&[0; 12]))
    }
}
