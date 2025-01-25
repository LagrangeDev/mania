use crate::core::tlv::prelude::*;

pub struct T119 {
    pub encrypted_tlv: Bytes,
}

impl TlvDe for T119 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            encrypted_tlv: p.bytes(),
        })))
    }

    impl_tlv_de!(0x119);
}
