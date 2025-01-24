use crate::event::prelude::*;
use crate::proto::sso_info_sync::*;

#[ce_commend("trpc.msg.register_proxy.RegisterProxy.SsoInfoSync")]
#[derive(Debug, ServerEvent)]
pub struct InfoSync;

impl ClientEvent for InfoSync {
    fn build(&self, ctx: &Context) -> BinaryPacket {
        let request = SsoInfoSyncRequest {
            SyncFlag: 735,
            ReqRandom: 298191, // FIXME:
            CurActiveStatus: 2,
            GroupLastMsgTime: 0,
            C2CInfoSync: ProtoMessageField::from(Some(SsoC2CInfoSync {
                C2CMsgCookie: ProtoMessageField::from(Some(SsoC2CMsgCookie {
                    C2CLastMsgTime: 0,
                    special_fields: Default::default(),
                })),
                C2CLastMsgTime: 0,
                LastC2CMsgCookie: ProtoMessageField::from(Some(SsoC2CMsgCookie {
                    C2CLastMsgTime: 0,
                    special_fields: Default::default(),
                })),
                special_fields: Default::default(),
            })),
            NormalConfig: ProtoMessageField::from(Some(NormalConfig {
                IntCfg: HashMap::new(),
                special_fields: Default::default(),
            })),
            RegisterInfo: ProtoMessageField::from(Some(RegisterInfo {
                Guid: hex::encode(ctx.device.uuid),
                KickPC: 0,
                CurrentVersion: ctx.app_info.current_version.parse().unwrap(),
                IsFirstRegisterProxyOnline: 1,
                LocaleId: 2052,
                Device: ProtoMessageField::from(Some(OnlineDeviceInfo {
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
                BusinessInfo: ProtoMessageField::from(Some(OnlineBusinessInfo {
                    NotifySwitch: 1,
                    BindUinNotifySwitch: 1,
                    special_fields: Default::default(),
                })),
                BatteryStatus: 0,
                Field12: 1,
                special_fields: Default::default(),
            })),
            UnknownStructure: ProtoMessageField::from(Some(UnknownStructure {
                GroupCode: 0,
                Flag2: 0,
                special_fields: Default::default(),
            })),
            AppState: ProtoMessageField::from(Some(CurAppState {
                IsDelayRequest: 0,
                AppStatus: 0,
                SilenceStatus: 0,
                special_fields: Default::default(),
            })),
            special_fields: Default::default(),
        };
        BinaryPacket(request.write_to_bytes().unwrap().into())
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        // TODO: parse InfoSyncRes
        Ok(Box::new(Self {}))
    }
}
