use crate::core::tlv::prelude::*;

pub struct T16A {
    pub no_pic_sig: Bytes,
}

impl TlvSer for T16A {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            no_pic_sig: ctx
                .key_store
                .session
                .no_pic_sig
                .load()
                .clone()
                .expect("no_pic_sig is none")
                .as_ref()
                .clone(),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x16A, |p| p.bytes(&self.no_pic_sig))
    }
}
