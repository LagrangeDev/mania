use crate::core::tlv::prelude::*;

pub struct T146 {
    pub state: u32,
    pub tag: String,
    pub message: String,
    pub field0: u32,
}

impl TlvDe for T146 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            state: p.u32(),
            tag: p.section_16_with_addition::<_, 0>(|p| p.string()),
            message: p.section_16_with_addition::<_, 0>(|p| p.string()),
            field0: p.u32(),
        })))
    }

    impl_tlv_de!(0x146);
}
