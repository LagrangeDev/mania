use crate::core::tlv::prelude::*;

pub struct T10A {
    pub tgt: Bytes,
}

impl TlvDe for T10A {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self { tgt: p.bytes() })))
    }

    impl_tlv_de!(0x10A);
}
