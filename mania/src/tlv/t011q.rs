use crate::tlv::prelude::*;

pub struct T011q {
    pub unusual_sign: Bytes,
}

impl TlvSer for T011q {
    fn from_context(_: &Context, pre: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            unusual_sign: Bytes::from(pre.unusual_sign.clone().expect("unusual sign is none")),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x011, |p| p.bytes(&self.unusual_sign))
    }
}
