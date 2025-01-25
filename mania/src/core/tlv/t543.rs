use crate::core::proto::tlv::Tlv543;
use crate::core::tlv::prelude::*;
use protobuf::Message;
pub struct T543 {
    pub uid: String,
}

impl TlvDe for T543 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| {
            let proto = Tlv543::parse_from_bytes(&p.bytes()).unwrap();
            Self {
                uid: proto.layer1.layer2.uid.clone(),
            }
        })))
    }

    impl_tlv_de!(0x543);
}
