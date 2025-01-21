use crate::tlv::prelude::*;

pub struct T144 {
    pub tgtgt_key: Bytes,
    pub tlvs: Bytes,
}

impl TlvSer for T144 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        let builder = PacketBuilder::new();
        let tlvs = serialize_tlv_set(ctx, &[0x16E, 0x147, 0x128, 0x124], builder).build();
        Box::new(Self {
            tgtgt_key: ctx.session.stub.tgtgt_key.load().as_ref().to_owned(),
            tlvs: Bytes::from(tlvs),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x144, |p| {
            let encrypted_tlvs = tea_encrypt(&self.tlvs, &self.tgtgt_key);
            p.bytes(&encrypted_tlvs)
        })
    }
}
