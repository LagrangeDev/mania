use crate::tlv::prelude::*;

pub struct T142 {
    pub package_name: String,
}

impl TlvSer for T142 {
    fn from_context(ctx: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            package_name: ctx.app_info.package_name.to_string(),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x142, |p| p.u16(0).string_with_length(&self.package_name))
    }
}
