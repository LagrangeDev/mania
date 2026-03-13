use crate::core::tlv::prelude::*;

pub struct T143 {
    pub d2: Bytes,
}

impl TlvDe for T143 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        Ok(Box::new(p.length_value(|p| Self { d2: p.bytes() })))
    }

    impl_tlv_de!(0x143);
}
