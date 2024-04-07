use crate::proto::tlv::{TlvQrCodeD1, TlvQrCodeD1Resp, NTOS};

use crate::tlv::prelude::*;

pub struct T0d1 {
    pub proto: TlvQrCodeD1,
}

impl TlvSer for T0d1 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            proto: TlvQrCodeD1 {
                sys: Some(NTOS {
                    name: ctx.device.device_name.clone(),
                    os: ctx.app_info.os.to_string(),
                    special_fields: Default::default(),
                })
                .into(),
                type_: vec![0x30, 0x01], // actually there is a type ext but i'm too lazy to implement it so i just hardcode it
                special_fields: Default::default(),
            },
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        // p.serialize_tlv(0x0d1, |p| p.serialize_proto(&self.proto))
        p.tlv(0x0d1, |p| p.proto(&self.proto))
    }
}

pub struct T0d1Resp {
    pub proto: TlvQrCodeD1Resp,
}

impl TlvDe for T0d1Resp {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        let proto = p.length_value(|p| p.proto())?;
        Ok(Box::new(Self { proto }))
    }

    impl_tlv_de!(0x0d1);
}
