use crate::core::entity::fetch_group_requests::FetchGroupRequests;
use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{OidbSvcTrpcTcp0x10C0, OidbSvcTrpcTcp0x10C0Response};

#[oidb_command(0x10c0, 2)]
#[derive(Debug, ServerEvent, Default)]
pub struct FetchFilteredGroupRequestsEvent {
    pub results: Vec<FetchGroupRequests>,
}

impl ClientEvent for FetchFilteredGroupRequestsEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let request = OidbSvcTrpcTcp0x10C0 {
            count: 20,
            field2: 0,
        };
        Ok(OidbPacket::new(0x10c0, 2, request.encode_to_vec(), false, false).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let response = OidbPacket::parse_into::<OidbSvcTrpcTcp0x10C0Response>(packet)?;
        let results = response
            .requests
            .into_iter()
            .map(|req| FetchGroupRequests {
                group_uin: req.group.as_ref().map_or(0, |g| g.group_uin),
                invitor_member_uid: req.invitor.as_ref().map(|i| i.uid.to_owned()),
                invitor_member_card: req.invitor.as_ref().map(|i| i.name.clone()),
                target_member_uid: req
                    .target
                    .as_ref()
                    .map_or("".to_string(), |t| t.uid.to_owned()),
                target_member_card: req
                    .target
                    .as_ref()
                    .map_or("".to_string(), |t| t.name.to_owned()),
                operator_uid: req.operator.as_ref().map(|o| o.uid.to_owned()),
                operator_name: req.operator.as_ref().map(|o| o.name.to_owned()),
                sequence: req.sequence,
                state: req.state,
                event_type: req.event_type,
                comment: Some(req.comment),
                is_filtered: true,
            })
            .collect::<Vec<_>>();
        Ok(ClientResult::single(Box::new(
            FetchFilteredGroupRequestsEvent { results },
        )))
    }
}
