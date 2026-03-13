use crate::core::tlv::prelude::*;

pub struct T01eq {
    pub tgtgt_key: Bytes,
}

impl TlvDe for T01eq {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            tgtgt_key: p.bytes(),
        })))
    }

    impl_tlv_de!(0x01e);
}
