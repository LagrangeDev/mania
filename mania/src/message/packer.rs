use crate::core::protos::message::{FileExtra, PushMsgBody};
use crate::dda;
use crate::entity::bot_friend::BotFriend;
use crate::entity::bot_group_member::BotGroupMember;
use crate::message::chain::{
    ClientSequence, FriendMessageUniqueElem, GroupMessageUniqueElem, MessageChain, MessageId,
    MessageType,
};
use crate::message::entity::Entity;
use crate::message::entity::file::{FileC2CUnique, FileEntity, FileUnique};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use prost::Message;

pub(crate) struct MessagePacker;

impl MessagePacker {
    pub(crate) fn parse_chain(push_msg_body: PushMsgBody) -> Result<MessageChain, String> {
        let response_head = push_msg_body.response_head.ok_or("missing ResponseHead")?;
        let content_head = push_msg_body
            .content_head
            .as_ref()
            .ok_or("missing ContentHead")?;
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
                .ok_or("failed to parse timestamp")?,
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
            .ok_or("failed to parse timestamp")?,
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
            .ok_or("missing response_head")?
            .grp
            .is_some();
        let typ = if is_group {
            MessageType::Group(
                make_group_extra(&body).ok_or_else(|| "failed to make_group_extra".to_string())?,
            )
        } else {
            MessageType::Friend(
                make_friend_extra(&body)
                    .ok_or_else(|| "failed to make_friend_extra".to_string())?,
            )
        };
        let mut chain = MessagePacker::parse_chain(body)?;
        chain.typ = typ;
        Ok(chain)
    }

    pub(crate) fn parse_private_file(body: PushMsgBody) -> Result<MessageChain, String> {
        let msg_content = body
            .body
            .as_ref()
            .and_then(|b| b.msg_content.clone())
            .ok_or_else(|| "missing msg_content".to_string())?;

        let mut base_chain = MessagePacker::parse_chain(body)?;
        let extra = FileExtra::decode(Bytes::from(msg_content))
            .map_err(|e| format!("failed to decode FileExtra: {:?}", e))?;
        let file = extra
            .file
            .as_ref()
            .ok_or_else(|| "missing file".to_string())?;
        match (
            file.file_size.as_ref(),
            file.file_name.as_ref(),
            file.file_md5.as_ref(),
            file.file_uuid.as_ref(),
            file.file_hash.as_ref(),
        ) {
            (
                Some(file_size),
                Some(file_name),
                Some(file_md5),
                Some(file_uuid),
                Some(file_hash),
            ) => base_chain.entities.push(Entity::File(dda!(FileEntity {
                file_size: *file_size as u64,
                file_name: file_name.to_owned(),
                file_md5: Bytes::from(file_md5.to_owned()),
                extra: Some(FileUnique::C2C(FileC2CUnique {
                    file_uuid: Some(file_uuid.to_owned()),
                    file_hash: Some(file_hash.to_owned()),
                })),
            }))),
            _ => return Err("missing file fields".to_string()),
        }
        Ok(base_chain)
    }
}
