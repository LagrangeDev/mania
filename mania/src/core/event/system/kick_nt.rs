use crate::core::event::prelude::*;
use crate::core::protos::kick_nt_response::*;

#[ce_commend("trpc.qq_new_tech.status_svc.StatusService.KickNT")]
#[derive(Debug, ServerEvent)]
pub struct KickNTEvent {
    tips: String,
    title: String,
}

impl ClientEvent for KickNTEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        unreachable!("KickNTEvent should not be sent by client")
    }

    fn parse(bytes: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError> {
        let res = ServiceKickNTResponse::parse_from_bytes(&bytes).map_err(|e| {
            ParseEventError::ParseError(format!("Failed to parse KickNTEvent: {:?}", e))
        })?;
        Ok(Box::new(Self {
            tips: res.Tips,
            title: res.Title,
        }))
    }
}
