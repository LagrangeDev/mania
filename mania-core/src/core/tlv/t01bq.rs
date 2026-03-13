use crate::core::tlv::prelude::*;

pub struct T01bq {
    pub micro: u32,
    pub version: u32,
    pub size: u32,
    pub margin: u32,
    pub dpi: u32,
    pub ec_level: u32,
    pub hint: u32,
    pub field0: u16,
}

impl TlvSer for T01bq {
    fn from_context(_: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            micro: 0,
            version: 0,
            size: 3,
            margin: 4,
            dpi: 72,
            ec_level: 2,
            hint: 2,
            field0: 0,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x01b, |p| {
            p.u32(self.micro)
                .u32(self.version)
                .u32(self.size)
                .u32(self.margin)
                .u32(self.dpi)
                .u32(self.ec_level)
                .u32(self.hint)
                .u16(self.field0)
        })
    }
}
