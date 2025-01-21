use crate::context::Context;
use crate::event::{ClientEvent, ParseEventError, ServerEvent};
use crate::packet::BinaryPacket;
use crate::proto::sso_info_sync::*;
use bytes::Bytes;
use protobuf::{Message, MessageField};
use std::collections::HashMap;

pub struct InfoSync;

#[derive(Debug)]
pub struct InfoSyncRes;

impl ServerEvent for InfoSyncRes {
    fn ret_code(&self) -> i32 {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ClientEvent for InfoSync {
    const COMMAND: &'static str = "trpc.msg.register_proxy.RegisterProxy.SsoInfoSync";

    fn build(&self, ctx: &Context) -> Vec<BinaryPacket> {
        let request = SsoInfoSyncRequest {
            SyncFlag: 735,
            ReqRandom: 298191, // FIXME:
            CurActiveStatus: 2,
            GroupLastMsgTime: 0,
            C2CInfoSync: MessageField::from(Some(SsoC2CInfoSync {
                C2CMsgCookie: MessageField::from(Some(SsoC2CMsgCookie {
                    C2CLastMsgTime: 0,
                    special_fields: Default::default(),
                })),
                C2CLastMsgTime: 0,
                LastC2CMsgCookie: MessageField::from(Some(SsoC2CMsgCookie {
                    C2CLastMsgTime: 0,
                    special_fields: Default::default(),
                })),
                special_fields: Default::default(),
            })),
            NormalConfig: MessageField::from(Some(NormalConfig {
                IntCfg: HashMap::new(),
                special_fields: Default::default(),
            })),
            RegisterInfo: MessageField::from(Some(RegisterInfo {
                Guid: hex::encode(ctx.device.uuid),
                KickPC: 0,
                CurrentVersion: ctx.app_info.current_version.parse().unwrap(),
                IsFirstRegisterProxyOnline: 1,
                LocaleId: 2052,
                Device: MessageField::from(Some(OnlineDeviceInfo {
                    User: ctx.device.device_name.clone(),
                    Os: ctx.app_info.kernel.to_string(),
                    OsVer: ctx.device.system_kernel.clone(),
                    VendorName: "".to_string(),
                    OsLower: ctx.app_info.vendor_os.to_string(),
                    special_fields: Default::default(),
                })),
                SetMute: 0,
                RegisterVendorType: 6,
                RegType: 0,
                BusinessInfo: MessageField::from(Some(OnlineBusinessInfo {
                    NotifySwitch: 1,
                    BindUinNotifySwitch: 1,
                    special_fields: Default::default(),
                })),
                BatteryStatus: 0,
                Field12: 1,
                special_fields: Default::default(),
            })),
            UnknownStructure: MessageField::from(Some(UnknownStructure {
                GroupCode: 0,
                Flag2: 0,
                special_fields: Default::default(),
            })),
            AppState: MessageField::from(Some(CurAppState {
                IsDelayRequest: 0,
                AppStatus: 0,
                SilenceStatus: 0,
                special_fields: Default::default(),
            })),
            special_fields: Default::default(),
        };
        vec![BinaryPacket(request.write_to_bytes().unwrap().into())]
    }

    fn parse(_: Bytes, _: &Context) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError> {
        // TODO: parse InfoSyncRes
        Ok(vec![Box::new(InfoSyncRes {})])
    }
}
