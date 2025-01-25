use crate::core::tlv::prelude::*;

pub struct T066q {
    pub pt_os_version: i32,
}

impl TlvSer for T066q {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            pt_os_version: ctx.app_info.pt_os_version,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x066, |p| p.i32(self.pt_os_version))
    }
}
