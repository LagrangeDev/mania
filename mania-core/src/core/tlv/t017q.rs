use crate::core::tlv::prelude::*;

pub struct T017q {
    pub qr_code: Bytes,
}

impl TlvDe for T017q {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        Ok(Box::new(p.length_value(|p| Self { qr_code: p.bytes() })))
    }

    impl_tlv_de!(0x017);
}
