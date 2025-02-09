use crate::core::event::prelude::*;
use crate::core::protos::system::{
    CurAppState, NormalConfig, OnlineBusinessInfo, OnlineDeviceInfo, RegisterInfo, SsoC2cInfoSync,
    SsoC2cMsgCookie, SsoInfoSyncRequest, UnknownStructure,
};
use crate::utility::random_gen::RandomGenerator;
use prost::Message;

#[command("trpc.msg.register_proxy.RegisterProxy.SsoInfoSync")]
#[derive(Debug, ServerEvent)]
pub struct InfoSyncEvent;

impl ClientEvent for InfoSyncEvent {
    fn build(&self, ctx: &Context) -> Result<BinaryPacket, EventError> {
        let request = SsoInfoSyncRequest {
            sync_flag: 735,
            req_random: RandomGenerator::rand_u32(),
            cur_active_status: 2,
            group_last_msg_time: 0,
            c2c_info_sync: Some(SsoC2cInfoSync {
                c2c_msg_cookie: Some(SsoC2cMsgCookie {
                    c2c_last_msg_time: 0,
                }),
                c2c_last_msg_time: 0,
                last_c2c_msg_cookie: Some(SsoC2cMsgCookie {
                    c2c_last_msg_time: 0,
                }),
            }),
            normal_config: Some(NormalConfig::default()),
            register_info: Some(RegisterInfo {
                guid: hex::encode(ctx.device.uuid),
                kick_pc: 0,
                current_version: ctx.app_info.current_version.parse().unwrap(),
                is_first_register_proxy_online: 1,
                locale_id: 2052,
                device: Some(OnlineDeviceInfo {
                    user: ctx.device.device_name.clone(),
                    os: ctx.app_info.kernel.to_string(),
                    os_ver: ctx.device.system_kernel.clone(),
                    vendor_name: "".to_string(),
                    os_lower: ctx.app_info.vendor_os.to_string(),
                }),
                set_mute: 0,
                register_vendor_type: 6,
                reg_type: 0,
                business_info: Some(OnlineBusinessInfo {
                    notify_switch: 1,
                    bind_uin_notify_switch: 1,
                }),
                battery_status: 0,
                field12: 1,
            }),
            unknown_structure: Some(UnknownStructure::default()),
            app_state: Some(CurAppState::default()),
        };
        Ok(BinaryPacket(request.encode_to_vec().into()))
    }

    fn parse(_: Bytes, _: &Context) -> CEParseResult {
        // TODO: parse InfoSyncRes
        Ok(ClientResult::single(Box::new(Self {})))
    }
}
