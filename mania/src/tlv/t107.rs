use crate::tlv::prelude::*;
pub struct T107 {
    pub pic_type: u16,
    pub cap_type: u8,
    pub pic_size: u16,
    pub pic_content: u8,
}

impl TlvSer for T107 {
    fn from_context(_: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            pic_type: 0x0001,
            cap_type: 0x0D,
            pic_size: 0x0000,
            pic_content: 0x01,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x107, |p| {
            p.u16(self.pic_type)
                .u8(self.cap_type)
                .u16(self.pic_size)
                .u8(self.pic_content)
        })
    }
}
