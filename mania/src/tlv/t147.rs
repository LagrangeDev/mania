use crate::tlv::prelude::*;

pub struct T147 {
    pub app_id: u32,
    pub pt_version: String,
    pub package_name: String,
}

impl TlvSer for T147 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            app_id: ctx.app_info.app_id as u32,
            pt_version: ctx.app_info.pt_version.parse().unwrap(),
            package_name: ctx.app_info.package_name.parse().unwrap(),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x147, |p| {
            p.u32(self.app_id)
                .string_with_length(&self.pt_version)
                .string_with_length(&self.package_name)
        })
    }
}
