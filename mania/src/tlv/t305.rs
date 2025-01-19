use crate::tlv::prelude::*;

pub struct T305 {
    pub d2key: Bytes,
}

impl TlvDe for T305 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self { d2key: p.bytes() })))
    }

    impl_tlv_de!(0x305);
}
