use crate::tlv::prelude::*;

pub struct T018q {
    pub temp_password: Bytes,
}

impl TlvDe for T018q {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            temp_password: p.bytes(),
        })))
    }

    impl_tlv_de!(0x018);
}
