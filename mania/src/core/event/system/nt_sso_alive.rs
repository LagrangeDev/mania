use crate::core::event::prelude::*;
use crate::core::protos::nt_sso_heartbeat::*;

#[ce_commend("trpc.qq_new_tech.status_svc.StatusService.SsoHeartBeat")]
#[derive(Debug, ServerEvent)]
pub struct NtSsoAliveEvent;

impl ClientEvent for NtSsoAliveEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let request = NTSsoHeartBeat {
            type_: 1,
            special_fields: Default::default(),
        };
        BinaryPacket(request.write_to_bytes().unwrap().into())
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        Ok(Box::new(Self {}))
    }
}
