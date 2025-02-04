use crate::core::protos::tlv::{Ntos, TlvQrCodeD1, TlvQrCodeD1Resp};
use crate::core::tlv::prelude::*;

pub struct T0d1q {
    pub proto: TlvQrCodeD1,
}

impl TlvSer for T0d1q {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            proto: TlvQrCodeD1 {
                sys: Some(Ntos {
                    name: ctx.device.device_name.clone(),
                    os: ctx.app_info.os.to_string(),
                }),
                r#type: vec![0x30, 0x01], // actually there is a type ext but i'm too lazy to implement it so i just hardcode it
            },
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x0d1, |p| p.proto(&self.proto))
    }
}

pub struct T0d1Resp {
    pub proto: TlvQrCodeD1Resp,
}

impl TlvDe for T0d1Resp {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        let proto = p.length_value(|p| p.proto())?;
        Ok(Box::new(Self { proto }))
    }

    impl_tlv_de!(0x0d1);
}
