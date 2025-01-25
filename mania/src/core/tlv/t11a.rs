use crate::core::packet::{PREFIX_NONE, PREFIX_U8};
use crate::core::tlv::prelude::*;

pub struct T11A {
    pub face_id: u16,
    pub age: u8,
    pub gender: u8,
    pub nick_name: String,
}

impl TlvDe for T11A {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| {
            let face_id = p.u16();
            let age = p.u8();
            let gender = p.u8();
            let nick_name = p.read_with_length::<_, { PREFIX_U8 | PREFIX_NONE }>(|p| p.string());
            Self {
                face_id,
                age,
                gender,
                nick_name,
            }
        })))
    }

    impl_tlv_de!(0x11A);
}
