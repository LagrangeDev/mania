use crate::tlv::prelude::*;

pub struct T019 {
    pub no_pic_sig: Bytes,
}

impl TlvDe for T019 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            no_pic_sig: p.bytes(),
        })))
    }

    impl_tlv_de!(0x019);
}
