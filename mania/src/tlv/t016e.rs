use crate::tlv::prelude::*;

pub struct T016E {
    pub device_name: String,
}

impl TlvSer for T016E {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            device_name: ctx.device.device_name.clone(),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x16E, |p| p.bytes(self.device_name.as_bytes()))
    }
}
