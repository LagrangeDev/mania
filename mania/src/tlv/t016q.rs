use crate::tlv::prelude::*;

pub struct T016q {
    pub sso_version: u32,
    pub appid: u32,
    pub sub_app_id: u32,
    pub app_id_qr_code: u32,
    pub guid: Uuid,
    pub package_name: &'static str,
    pub pt_version: &'static str,
}

impl TlvSer for T016q {
    fn from_context(ctx: &Context, _: &TlvPreload) -> Box<dyn TlvSer> {
        Box::new(Self {
            sso_version: 0,
            appid: ctx.app_info.app_id as u32,
            sub_app_id: ctx.app_info.sub_app_id as u32,
            app_id_qr_code: ctx.app_info.app_id_qr_code as u32,
            guid: ctx.device.uuid,
            package_name: ctx.app_info.package_name,
            pt_version: ctx.app_info.pt_version,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x016, |p| {
            p.u32(0)
                .u32(self.appid)
                .u32(self.sub_app_id)
                .bytes(self.guid.to_bytes())
                .string_with_length(self.package_name)
                .string_with_length(self.pt_version)
                .string_with_length(self.package_name)
        })
    }
}
