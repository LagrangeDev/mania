use crate::core::event::prelude::*;
use crate::core::protos::system::ServiceKickNtResponse;

#[commend("trpc.qq_new_tech.status_svc.StatusService.KickNT")]
#[derive(Debug, ServerEvent)]
pub struct KickNTEvent {
    tips: String,
    title: String,
}

impl ClientEvent for KickNTEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        unreachable!("KickNTEvent should not be sent by client")
    }

    fn parse(mut bytes: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        let res = ServiceKickNtResponse::decode(&mut bytes).map_err(|e| {
            ParseEventError::ProtoParseError(format!("Failed to parse KickNTEvent: {:?}", e))
        })?;
        Ok(Box::new(Self {
            tips: res.tips,
            title: res.title,
        }))
    }
}
