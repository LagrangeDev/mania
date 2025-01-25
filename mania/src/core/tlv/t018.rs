use crate::core::tlv::prelude::*;

pub struct T018 {
    pub ping_version: u16,
    pub sso_version: u32,
    pub app_id: u32,
    pub app_client_version: u32,
    pub uin: u32,
    pub field0: u16,
    pub field1: u16,
}

impl TlvSer for T018 {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer> {
        Box::new(Self {
            ping_version: 0,
            sso_version: 0x00000005,
            app_id: 0,
            app_client_version: 30366,
            uin: **ctx.key_store.uin.load(),
            field0: 0,
            field1: 0,
        })
    }

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder {
        p.tlv(0x18, |p| {
            p.u16(self.ping_version)
                .u32(self.sso_version)
                .u32(self.app_id)
                .u32(self.app_client_version)
                .u32(self.uin)
                .u16(self.field0)
                .u16(self.field1)
        })
    }
}
