use crate::core::tlv::prelude::*;

pub struct T100 {
    pub db_buf_version: u16,
    pub sso_version: u32,
    pub appid: u32,
    pub sub_app_id: u32,
    pub app_client_version: u32,
    pub main_sigmap: u32,
}

impl TlvSer for T100 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            db_buf_version: 0,
            sso_version: 0x00000005,
            appid: ctx.app_info.app_id as u32,
            sub_app_id: ctx.app_info.sub_app_id as u32,
            app_client_version: ctx.app_info.app_client_version as u32,
            main_sigmap: ctx.app_info.main_sig_map,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x100, |p| {
            p.u16(self.db_buf_version)
                .u32(self.sso_version)
                .u32(self.appid)
                .u32(self.sub_app_id)
                .u32(self.app_client_version)
                .u32(self.main_sigmap)
        })
    }
}
