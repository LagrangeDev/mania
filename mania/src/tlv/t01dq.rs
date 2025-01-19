use crate::tlv::prelude::*;

pub struct T01dq {
    pub field_byte: u8,
    pub misc_bitmap: u32,
    pub field0: u32,
    pub field_byte0: u8,
}

impl TlvSer for T01dq {
    fn from_context(ctx: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            field_byte: 1,
            misc_bitmap: ctx.app_info.misc_bitmap as u32,
            field0: 0,
            field_byte0: 0,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x01d, |p| {
            p.u8(self.field_byte)
                .u32(self.misc_bitmap)
                .u32(self.field0)
                .u8(self.field_byte0)
        })
    }
}
