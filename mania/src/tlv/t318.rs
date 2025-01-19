use crate::tlv::prelude::*;

pub struct T318 {
    pub tgt_qr: u8,
}

impl TlvSer for T318 {
    fn from_context(_: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self { tgt_qr: 0 })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x318, |p| p)
    }
}
