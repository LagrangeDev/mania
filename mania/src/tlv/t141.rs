use crate::tlv::prelude::*;

pub struct T141 {
    pub version: u16,
    pub unknown: String,
    pub network_type: u16,
    pub apn: String,
}

impl TlvSer for T141 {
    fn from_context(_: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            version: 0,
            unknown: "Unknown".to_string(),
            network_type: 0,
            apn: "".to_string(),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x141, |p| {
            p.u16(self.version)
                .string_with_length(&self.unknown)
                .u16(self.network_type)
                .string_with_length(&self.apn)
        })
    }
}
