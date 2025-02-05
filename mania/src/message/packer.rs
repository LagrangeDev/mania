use crate::core::protos::message::PushMsgBody;
use crate::dda;
use crate::entity::bot_friend::BotFriend;
use crate::entity::bot_group_member::BotGroupMember;
use crate::message::chain::{
    ClientSequence, FriendMessageUniqueElem, GroupMessageUniqueElem, MessageChain, MessageId,
    MessageType,
};
use crate::message::entity::Entity;
use chrono::{DateTime, Utc};

pub(crate) struct MessagePacker;

impl MessagePacker {
    pub(crate) fn parse_chain(push_msg_body: PushMsgBody) -> Result<MessageChain, String> {
        let response_head = push_msg_body.response_head.expect("missing ResponseHead");
        let content_head = push_msg_body
            .content_head
            .as_ref()
            .expect("missing ContentHead");

        let pre_len = push_msg_body
            .body
            .as_ref()
            .and_then(|body| body.rich_text.as_ref())
            .map_or(0, |rich_text| rich_text.elems.len());
        let mut entities: Vec<Entity> = Vec::with_capacity(pre_len);
        if let Some(rich_text) = push_msg_body
            .body
            .as_ref()
            .and_then(|body| body.rich_text.as_ref())
        {
            entities.extend(rich_text.elems.iter().filter_map(Entity::unpack_element));
        }

        if let Some(grp) = response_head.grp {
            return Ok(dda!(MessageChain {
                typ: MessageType::Group(dda!(GroupMessageUniqueElem {
                    group_uin: grp.group_uin,
                })),
                friend_uin: response_head.from_uin,
                message_id: MessageId(content_head.msg_uid.unwrap_or_default()),
                time: DateTime::<Utc>::from_timestamp(
                    content_head.time_stamp.unwrap_or_default() as i64,
                    0
                )
                .unwrap(),
                sequence: content_head.sequence.unwrap_or_default(),
                entities: entities,
            }));
        }
        Ok(dda!(MessageChain {
            typ: match content_head.r#type {
                141 => MessageType::Temp,
                _ => MessageType::Friend(FriendMessageUniqueElem {
                    friend_info: None,
                    client_sequence: ClientSequence(content_head.sequence.unwrap_or_default()),
                }),
            },
            uid: response_head.from_uid.unwrap_or_default(),
            self_uid: response_head.to_uid.unwrap_or_default(),
            target_uin: response_head.to_uin,
            friend_uin: response_head.from_uin,
            message_id: MessageId(content_head.msg_uid.unwrap_or_default()),
            time: DateTime::<Utc>::from_timestamp(
                content_head.time_stamp.unwrap_or_default() as i64,
                0
            )
            .unwrap(),
            sequence: content_head.nt_msg_seq.unwrap_or_default(),
            entities: entities,
        }))
    }

    pub(crate) fn parse_fake_chain(body: PushMsgBody) -> Result<MessageChain, String> {
        let make_group_extra = |body: &PushMsgBody| -> Option<GroupMessageUniqueElem> {
            Some(GroupMessageUniqueElem {
                group_uin: body.response_head.as_ref()?.grp.as_ref()?.group_uin,
                group_member_info: Some(dda!(BotGroupMember {
                    member_card: Some(
                        body.response_head
                            .as_ref()?
                            .grp
                            .as_ref()?
                            .member_name
                            .clone()
                    ),
                    member_name: body
                        .response_head
                        .as_ref()?
                        .grp
                        .as_ref()?
                        .member_name
                        .clone(),
                    uid: body
                        .response_head
                        .as_ref()?
                        .from_uid
                        .clone()
                        .unwrap_or_default(),
                })),
            })
        };
        let make_friend_extra = |body: &PushMsgBody| -> Option<FriendMessageUniqueElem> {
            Some(FriendMessageUniqueElem {
                client_sequence: ClientSequence(
                    body.content_head.as_ref()?.sequence.unwrap_or_default(),
                ),
                friend_info: Some(dda!(BotFriend {
                    nickname: body
                        .response_head
                        .as_ref()?
                        .from_uid
                        .clone()
                        .unwrap_or_default(),
                })),
            })
        };
        let is_group = body
            .response_head
            .as_ref()
            .expect("missing response_head")
            .grp
            .is_some();
        let typ = if is_group {
            MessageType::Group(make_group_extra(&body).unwrap())
        } else {
            MessageType::Friend(make_friend_extra(&body).unwrap())
        };
        let mut chain = MessagePacker::parse_chain(body)?;
        chain.typ = typ;
        Ok(chain)
    }
}
