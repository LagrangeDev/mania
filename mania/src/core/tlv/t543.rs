use crate::core::protos::tlv::Tlv543;
use crate::core::tlv::prelude::*;

pub struct T543 {
    pub uid: String,
}

impl TlvDe for T543 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        Ok(Box::new(p.length_value(|p| {
            let proto = Tlv543::decode(&mut p.bytes()).unwrap();
            Self {
                uid: proto.layer1.unwrap().layer2.unwrap().uid.clone(),
            }
        })))
    }

    impl_tlv_de!(0x543);
}
