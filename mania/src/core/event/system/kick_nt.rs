use crate::core::event::prelude::*;
use crate::core::protos::system::ServiceKickNtResponse;

#[command("trpc.qq_new_tech.status_svc.StatusService.KickNT")]
#[derive(Debug, ServerEvent)]
pub struct KickNTEvent {
    pub tips: String,
    pub title: String,
}

impl ClientEvent for KickNTEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        unreachable!("KickNTEvent should not be sent by client")
    }

    fn parse(mut bytes: Bytes, _: &Context) -> CEParseResult {
        let res = ServiceKickNtResponse::decode(&mut bytes)?;
        Ok(ClientResult::single(Box::new(Self {
            tips: res.tips,
            title: res.title,
        })))
    }
}
