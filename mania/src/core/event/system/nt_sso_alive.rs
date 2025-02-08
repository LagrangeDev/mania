use crate::core::event::prelude::*;
use crate::core::protos::system::NtSsoHeartBeat;

#[command("trpc.qq_new_tech.status_svc.StatusService.SsoHeartBeat")]
#[derive(Debug, ServerEvent)]
pub struct NtSsoAliveEvent;

impl ClientEvent for NtSsoAliveEvent {
    fn build(&self, _: &Context) -> Result<BinaryPacket, EventError> {
        let request = NtSsoHeartBeat { r#type: 1 };
        Ok(BinaryPacket(request.encode_to_vec().into()))
    }

    fn parse(_: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        Ok(Box::new(Self {})) // TODO: parse
    }
}
