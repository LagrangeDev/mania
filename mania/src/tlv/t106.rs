use crate::tlv::prelude::*;

// FIXME: dummy T106
pub struct T106 {
    pub app_id: i32,
    pub app_client_version: i32,
    pub uin: i32,
    pub password_md5: Bytes,
    pub guid: String,
    pub tgtgt_key: [u8; 16],
    pub ip: [u8; 4],
    pub save_password: bool,
    pub temp: Bytes,
}

impl TlvSer for T106 {
    fn from_context(ctx: &Context, pre: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            app_id: ctx.app_info.app_id,
            app_client_version: ctx.app_info.app_client_version as i32,
            uin: 0,
            password_md5: Bytes::default(),
            guid: hex::encode(ctx.device.uuid),
            tgtgt_key: [0; 16],
            ip: [0, 0, 0, 0],
            save_password: true,
            temp: pre.temp_password.clone().expect("temp password not found"),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        tracing::warn!("T106: &self.temp = {}", hex::encode(&self.temp));
        p.tlv(0x106, |p| p.bytes(&self.temp))
    }
}

impl TlvDe for T106 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            app_id: 0,
            app_client_version: 0,
            uin: 0,
            password_md5: Default::default(),
            guid: "".to_string(),
            tgtgt_key: [0; 16],
            ip: [0; 4],
            save_password: false,
            temp: p.bytes(),
        })))
    }

    impl_tlv_de!(0x106);
}
