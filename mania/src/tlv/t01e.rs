use crate::tlv::prelude::*;

pub struct T01e {
    pub tgtgt_key: Bytes,
}

impl TlvDe for T01e {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            tgtgt_key: p.bytes(),
        })))
    }

    impl_tlv_de!(0x01e);
}
