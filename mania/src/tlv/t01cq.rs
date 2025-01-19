use crate::tlv::prelude::*;

pub struct T01cq {
    pub expire_sec: u32,
    pub expire_min: u16,
}

impl TlvDe for T01cq {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            expire_sec: p.u32(),
            expire_min: p.u16(),
        })))
    }

    impl_tlv_de!(0x01c);
}
