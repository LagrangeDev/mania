use crate::tlv::prelude::*;

pub struct T16A {
    pub no_pic_sig: Bytes,
}

impl TlvSer for T16A {
    fn from_context(_: &Context, pre: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            no_pic_sig: pre.no_pic_sig.clone().expect("no_pic_sig is none"),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x16A, |p| p.bytes(&self.no_pic_sig))
    }
}
