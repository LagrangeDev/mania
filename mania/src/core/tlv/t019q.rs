use crate::core::tlv::prelude::*;

pub struct T019q {
    pub no_pic_sig: Bytes,
}

impl TlvDe for T019q {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            no_pic_sig: p.bytes(),
        })))
    }

    impl_tlv_de!(0x019);
}
