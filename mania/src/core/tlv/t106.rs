use crate::core::tlv::prelude::*;

// FIXME: dummy T106
pub struct T106 {
    pub app_id: i32,
    pub app_client_version: i32,
    pub uin: i32,
    pub password_md5: Bytes,
    pub guid: String,
    pub tgtgt_key: Bytes,
    pub ip: [u8; 4],
    pub save_password: bool,
    pub temp: Bytes,
}

impl TlvSer for T106 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            app_id: ctx.app_info.app_id,
            app_client_version: ctx.app_info.app_client_version as i32,
            uin: **ctx.key_store.uin.load() as i32,
            password_md5: ctx.key_store.password_md5.load().as_ref().to_owned(),
            guid: hex::encode(ctx.device.uuid),
            tgtgt_key: ctx.session.stub.tgtgt_key.load().as_ref().to_owned(),
            ip: [0, 0, 0, 0],
            save_password: true,
            temp: ctx
                .key_store
                .session
                .temp_password
                .load_full()
                .as_ref()
                .map(|arc| (**arc).clone())
                .expect("Missing temp password"),
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x106, |p| p.bytes(&self.temp))
    }
}

impl TlvDe for T106 {
    fn deserialize(p: &mut PacketReader) -> Result<Box<dyn TlvDe>, TlvError> {
        Ok(Box::new(p.length_value(|p| Self {
            app_id: 0,
            app_client_version: 0,
            uin: 0,
            password_md5: Default::default(),
            guid: "".to_string(),
            tgtgt_key: Bytes::new(),
            ip: [0; 4],
            save_password: false,
            temp: p.bytes(),
        })))
    }

    impl_tlv_de!(0x106);
}
