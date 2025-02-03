use crate::core::event::ParseEventError;
use crate::core::protos::message::PushMsgBody;
use crate::dda;
use crate::message::chain::{
    ClientSequence, FriendMessageUniqueElem, GroupMessageUniqueElem, MessageChain, MessageId,
    MessageType,
};
use crate::message::entity::Entity;
use chrono::{DateTime, Utc};

pub(crate) struct MessagePacker;

impl MessagePacker {
    pub(crate) fn parse_chain(push_msg_body: PushMsgBody) -> Result<MessageChain, ParseEventError> {
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

    fn parse_fake_chain(_: PushMsgBody) -> Result<MessageChain, ParseEventError> {
        todo!()
    }
}
