use crate::tlv::prelude::*;

pub struct T128 {
    pub app_info_os: String,
    pub device_guid: Uuid,
}

impl TlvSer for T128 {
    fn from_context(ctx: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            app_info_os: ctx.app_info.os.to_string(),
            device_guid: ctx.device.uuid,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x128, |p| {
            p.u16(0)
                .u8(0)
                .u8(1)
                .u8(0)
                .u32(0)
                .string_with_length(&self.app_info_os)
                .bytes_with_length(self.device_guid.to_bytes())
                .string_with_length("")
        })
    }
}
