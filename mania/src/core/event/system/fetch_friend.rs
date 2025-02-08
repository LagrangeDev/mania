use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    OidbNumber, OidbSvcTrpcTcp0xFd41, OidbSvcTrpcTcp0xFd41body, OidbSvcTrpcTcp0xFd41response,
    OidbSvcTrpcTcp0xFd41uin,
};
use crate::entity::bot_friend::{BotFriend, BotFriendGroup};
use std::collections::HashMap;

#[oidb_command(0xfd4, 1)]
#[derive(Debug, ServerEvent, Default)]
pub struct FetchFriendsEvent {
    pub friends: Vec<BotFriend>,
    pub friend_groups: HashMap<u32, String>,
    pub next_uin: Option<u32>,
}

impl ClientEvent for FetchFriendsEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        let request = OidbSvcTrpcTcp0xFd41 {
            field2: 300,
            field4: 0,
            next_uin: self.next_uin.map(|uin| OidbSvcTrpcTcp0xFd41uin { uin }),
            field6: 1,
            body: vec![
                OidbSvcTrpcTcp0xFd41body {
                    r#type: 1,
                    number: Some(OidbNumber {
                        numbers: vec![103, 102, 20002, 27394],
                    }),
                },
                OidbSvcTrpcTcp0xFd41body {
                    r#type: 4,
                    number: Some(OidbNumber {
                        numbers: vec![100, 101, 102],
                    }),
                },
            ],
            field10002: vec![13578, 13579, 13573, 13572, 13568],
            field10003: 4051,
        };
        OidbPacket::new(0xfd4, 1, request.encode_to_vec(), false, false).to_binary()
    }

    fn parse(packet: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        let response = OidbPacket::parse_into::<OidbSvcTrpcTcp0xFd41response>(packet)?;
        let mut friends = Vec::with_capacity(response.friends.len());
        for friend in &response.friends {
            let additional = friend
                .additional
                .iter()
                .find(|a| a.r#type == 1)
                .ok_or_else(|| EventError::OtherError("Missing additional with type 1".into()))?;
            let layer1 = additional
                .layer1
                .as_ref()
                .ok_or_else(|| EventError::OtherError("Missing layer1 in additional".into()))?;
            let mut display_name = None;
            let mut remark = None;
            let mut signature = None;
            let mut qid = None;
            for p in &layer1.properties {
                match p.code {
                    20002 => display_name = Some(p.value.as_str()),
                    103 => remark = Some(p.value.as_str()),
                    102 => signature = Some(p.value.as_str()),
                    27394 => qid = Some(p.value.as_str()),
                    _ => {}
                }
            }
            let display_name = display_name.map_or_else(|| friend.uin.to_string(), str::to_owned);
            let bot_friend = BotFriend::new(
                friend.uin,
                friend.uid.to_owned(),
                display_name,
                remark.unwrap_or("").to_owned(),
                signature.unwrap_or("").to_owned(),
                qid.unwrap_or("").to_owned(),
                Some(dda!(BotFriendGroup {
                    group_id: friend.custom_group,
                })),
            );
            friends.push(bot_friend);
        }
        Ok(Box::new(FetchFriendsEvent {
            friends,
            friend_groups: response
                .groups
                .iter()
                .map(|group| (group.code, group.value.to_owned()))
                .collect(),
            next_uin: response.next.map(|n| n.uin),
        }))
    }
}
