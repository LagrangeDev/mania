use crate::core::entity::fetch_group_requests::FetchGroupRequests;
use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{OidbSvcTrpcTcp0x10C0, OidbSvcTrpcTcp0x10C0Response};

#[oidb_command(0x10c0, 1)]
#[derive(Debug, ServerEvent, Default)]
pub struct FetchGroupRequestsEvent {
    pub results: Vec<FetchGroupRequests>,
}

impl ClientEvent for FetchGroupRequestsEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let request = OidbSvcTrpcTcp0x10C0 {
            count: 20,
            field2: 0,
        };
        Ok(OidbPacket::new(0x10c0, 1, request.encode_to_vec(), false, false).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let response = OidbPacket::parse_into::<OidbSvcTrpcTcp0x10C0Response>(packet)?;
        let results = response
            .requests
            .into_iter()
            .map(|mut req| FetchGroupRequests {
                group_uin: req.group.take().map_or(0, |g| g.group_uin),
                invitor_member_uid: req.invitor.take().map(|i| i.uid),
                invitor_member_card: req.invitor.take().map(|i| i.name),
                target_member_uid: req.target.take().map_or("".to_string(), |t| t.uid),
                target_member_card: req.target.take().map_or("".to_string(), |t| t.name),
                operator_uid: req.operator.take().map(|o| o.uid),
                operator_name: req.operator.take().map(|o| o.name),
                sequence: req.sequence,
                state: req.state,
                event_type: req.event_type,
                comment: Some(req.comment),
                is_filtered: false,
            })
            .collect::<Vec<_>>();
        Ok(ClientResult::single(Box::new(FetchGroupRequestsEvent {
            results,
        })))
    }
}
