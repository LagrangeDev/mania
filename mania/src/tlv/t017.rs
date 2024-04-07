use crate::tlv::prelude::*;

pub struct T017 {
    pub qr_code: Bytes,
}

impl TlvDe for T017 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self { qr_code: p.bytes() })))
    }

    impl_tlv_de!(0x017);
}
