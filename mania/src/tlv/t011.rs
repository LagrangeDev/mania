use crate::tlv::prelude::*;

pub struct T011 {
    pub unusual_sign: Vec<u8>,
}

impl TlvSer for T011 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            unusual_sign: ctx
                .session
                .unusual_sign
                .clone()
                .expect("unusual sign is none"),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x011, |p| p.bytes(&self.unusual_sign))
    }
}
