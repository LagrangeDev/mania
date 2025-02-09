use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    OidbSvcTrpcScp0xFe7Body, OidbSvcTrpcTcp0xFe7Level, OidbSvcTrpcTcp0xFe73,
    OidbSvcTrpcTcp0xFe73response,
};
use crate::entity::bot_group_member::{BotGroupMember, GroupMemberPermission};
use chrono::DateTime;

#[oidb_command(0xfe7, 3)]
#[derive(Debug, ServerEvent, Default)]
pub struct FetchMembersEvent {
    pub group_uin: u32,
    pub group_members: Vec<BotGroupMember>,
    pub token: Option<String>,
}

impl ClientEvent for FetchMembersEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let request = OidbSvcTrpcTcp0xFe73 {
            group_uin: self.group_uin,
            field2: 5,
            field3: 2,
            body: Some(dda!(OidbSvcTrpcScp0xFe7Body {
                member_name: true,
                member_card: true,
                level: true,
                special_title: true,
                join_timestamp: true,
                last_msg_timestamp: true,
                shut_up_timestamp: true,
                permission: true,
            })),
            token: self.token.clone(),
        };
        Ok(OidbPacket::new(0xfe7, 3, request.encode_to_vec(), false, true).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let response = OidbPacket::parse_into::<OidbSvcTrpcTcp0xFe73response>(packet)?;
        let to_dt = |ts: u32, err_msg: &'static str| {
            DateTime::from_timestamp(ts as i64, 0)
                .ok_or_else(|| EventError::OtherError(err_msg.into()))
        };
        let group_members = response
            .members
            .into_iter()
            .map(|mut member| {
                let uin_data = member
                    .uin
                    .take()
                    .ok_or_else(|| EventError::OtherError("Missing uin".into()))?;
                let card = member.member_card.and_then(|mut mc| mc.member_card.take());
                Ok::<BotGroupMember, EventError>(BotGroupMember {
                    uin: uin_data.uin,
                    uid: uin_data.uid,
                    permission: GroupMemberPermission::try_from(member.permission)
                        .map_err(|_| EventError::OtherError("Invalid permission".into()))?,
                    group_level: member
                        .level
                        .map(|l| l.level)
                        .unwrap_or_else(|| OidbSvcTrpcTcp0xFe7Level::default().level),
                    member_card: card,
                    member_name: member.member_name,
                    special_title: member.special_title,
                    join_time: to_dt(member.join_timestamp, "Invalid join timestamp")?,
                    last_msg_time: to_dt(
                        member.last_msg_timestamp,
                        "Invalid last message timestamp",
                    )?,
                    shut_up_timestamp: to_dt(
                        member.shut_up_timestamp.unwrap_or(0),
                        "Invalid shut up timestamp",
                    )?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ClientResult::single(Box::new(FetchMembersEvent {
            group_uin: response.group_uin,
            group_members,
            token: response.token,
        })))
    }
}
