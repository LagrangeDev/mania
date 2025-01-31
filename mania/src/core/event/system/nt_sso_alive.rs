use crate::core::event::prelude::*;
use crate::core::protos::system::NtSsoHeartBeat;

#[ce_commend("trpc.qq_new_tech.status_svc.StatusService.SsoHeartBeat")]
#[derive(Debug, ServerEvent)]
pub struct NtSsoAliveEvent;

impl ClientEvent for NtSsoAliveEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let request = NtSsoHeartBeat { r#type: 1 };
        BinaryPacket(request.encode_to_vec().into())
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        Ok(Box::new(Self {}))
    }
}
