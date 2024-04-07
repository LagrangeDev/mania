use crate::tlv::prelude::*;

pub struct T016 {
    pub sso_version: u32,
    pub sub_app_id: u32,
    pub app_id_qr_code: u32,
    pub guid: Vec<u8>,
    pub package_name: &'static str,
    pub pt_version: &'static str,
    pub package_name2: &'static str,
}

impl TlvSer for T016 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            sso_version: 0,
            sub_app_id: ctx.app_info.app_id as u32,
            app_id_qr_code: ctx.app_info.app_id_qr_code as u32,
            guid: ctx.device.uuid.to_bytes_le().to_vec(),
            package_name: ctx.app_info.package_name,
            pt_version: ctx.app_info.pt_version,
            package_name2: ctx.app_info.package_name,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x016, |p| {
            p.u32(self.sso_version)
                .u32(self.sub_app_id)
                .u32(self.app_id_qr_code)
                .bytes(&self.guid)
                .string_with_length(self.package_name)
                .string_with_length(self.pt_version)
                .string_with_length(self.package_name2)
        })
    }
}
