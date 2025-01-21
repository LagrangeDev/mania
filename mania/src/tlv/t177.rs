use crate::tlv::prelude::*;

pub struct T177 {
    pub field_1: Bytes,
    pub build_time: u32,
    pub wt_login_sdk: String,
}

impl TlvSer for T177 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            field_1: Bytes::from_static(b"\x01"),
            build_time: 0,
            wt_login_sdk: ctx.app_info.wt_login_sdk.to_string(),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x177, |p| {
            p.u8(1)
                .u32(self.build_time)
                .string_with_length(&self.wt_login_sdk)
        })
    }
}
