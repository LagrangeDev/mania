use crate::tlv::prelude::*;
pub struct T033 {
    pub guid: Vec<u8>,
}

impl TlvSer for T033 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            guid: ctx.device.uuid.to_bytes_le().to_vec(),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x033, |p| p.bytes(&self.guid))
    }
}
