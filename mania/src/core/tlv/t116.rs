use crate::core::tlv::prelude::*;

pub struct T116 {
    pub sub_sigmap: u16,
}

impl TlvSer for T116 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            sub_sigmap: ctx.app_info.sub_sig_map,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x116, |p| {
            p.u8(0).u32(12058620).u32(self.sub_sigmap as u32).u8(0)
        })
    }
}
