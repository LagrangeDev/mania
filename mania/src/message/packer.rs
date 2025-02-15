use crate::core::protos::message::{
    C2c, ContentHead, FileExtra, Grp, Message, MessageBody, MessageControl, PushMsgBody, RichText,
    RoutingHead, Trans0X211,
};
use crate::entity::bot_friend::BotFriend;
use crate::entity::bot_group_member::{BotGroupMember, FetchGroupMemberStrategy};
use crate::message::chain::{
    ClientSequence, FriendMessageUniqueElem, GroupMessageUniqueElem, MessageChain, MessageId,
    MessageType,
};
use crate::message::entity::Entity;
use crate::message::entity::file::{FileC2CUnique, FileEntity, FileUnique};
use crate::{Context, dda};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use prost::Message as _;

pub(crate) struct MessagePacker;

impl MessagePacker {
    #[allow(clippy::let_and_return)] // FIXME: remove this
    pub(crate) fn build(chain: &MessageChain, self_uid: &str) -> Message {
        let base = MessagePacker::build_packet_base(chain, self_uid);
        // TODO: BuildAdditional(chain, message);
        base
    }

    fn build_packet_base(chain: &MessageChain, self_uid: &str) -> Message {
        dda!(Message {
            routing_head: Some(dda!(RoutingHead {
                c2c: match (&chain.typ, Entity::need_pack_content(&chain.entities)) {
                    (MessageType::Friend(friend), false) => Some(dda!(C2c {
                        uin: Some(chain.friend_uin),
                        uid: friend.friend_info.as_ref().map(|f| f.uid.clone()),
                    })),
                    _ => None,
                },
                grp: match &chain.typ {
                    MessageType::Group(group) => Some(Grp {
                        group_code: Some(group.group_uin),
                    }),
                    _ => None,
                },
                trans0_x211: match Entity::need_pack_content(&chain.entities) {
                    true => Some(dda!(Trans0X211 {
                        cc_cmd: Some(4),
                        uid: Some(chain.uid.clone()),
                    })),
                    false => None,
                },
            })),
            content_head: Some(dda!(ContentHead {
                r#type: 1,
                sub_type: Some(0),
                c2c_cmd: Some(0),
            })),
            body: Some(dda!(MessageBody {
                rich_text: Some(dda!(RichText {
                    elems: chain
                        .entities
                        .iter()
                        .flat_map(|entity| entity.pack_element(self_uid))
                        .collect(),
                })),
                msg_content: chain
                    .entities
                    .iter()
                    .filter_map(Entity::pack_content)
                    .next()
                    .map(|c| { c.to_vec() }),
            })),
            client_sequence: match &chain.typ {
                MessageType::Friend(_) => Some(chain.sequence),
                _ => Some(0),
            },
            random: Some((chain.message_id.0 & 0xFFFFFFFF) as u32),
            ctrl: match &chain.typ {
                MessageType::Friend(_) => Some(MessageControl {
                    msg_flag: Utc::now().timestamp() as i32,
                }),
                _ => None,
            },
        })
    }

    pub(crate) fn parse_chain(
        push_msg_body: PushMsgBody,
        ctx: &Context,
    ) -> Result<MessageChain, String> {
        let response_head = push_msg_body
            .response_head
            .as_ref()
            .ok_or("missing ResponseHead")?;
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
        if let Some(grp) = &response_head.grp {
            let (mut ex_gf, mut ex_info) = match ctx.config.fetch_group_member_strategy {
                FetchGroupMemberStrategy::Simple => {
                    let mut extra_entities = Vec::with_capacity(2);
                    if let Some(rich_text) = push_msg_body
                        .body
                        .as_ref()
                        .and_then(|body| body.rich_text.as_ref())
                    {
                        extra_entities.extend(
                            rich_text
                                .elems
                                .iter()
                                .filter_map(Entity::unpack_extra_element),
                        );
                    }
                    extra_entities
                        .into_iter()
                        .fold((None, None), |(gf, info), e| match e {
                            Entity::ExtraGeneralFlags(inner) => (Some(inner), info),
                            Entity::ExtraInfo(inner) => (gf, Some(inner)),
                            _ => (gf, info),
                        })
                }
                _ => (None, None),
            };
            return Ok(dda!(MessageChain {
                typ: MessageType::Group(GroupMessageUniqueElem {
                    group_uin: grp.group_code.unwrap_or_default() as u32,
                    group_member_info: match ctx.config.fetch_group_member_strategy {
                        FetchGroupMemberStrategy::Simple => {
                            let res_head = push_msg_body
                                .response_head
                                .as_ref()
                                .ok_or("missing response_head")?;
                            let grp = res_head.grp.as_ref().ok_or("missing grp")?;
                            Some(dda!(BotGroupMember {
                                uin: res_head.from_uin,
                                uid: res_head.from_uid.to_owned().unwrap_or_default(),
                                permission: ex_gf
                                    .as_mut()
                                    .map(|gf| std::mem::take(&mut gf.permission))
                                    .unwrap_or_default(),
                                group_level: ex_gf
                                    .as_ref()
                                    .map(|gf| gf.new_group_level)
                                    .unwrap_or_default(),
                                member_card: grp.group_card.to_owned(),
                                special_title: ex_info
                                    .as_mut()
                                    .and_then(|info| info.group_member_special_title.take()),
                            }))
                        }
                        _ => None,
                    }
                }),
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
            uid: response_head.from_uid.to_owned().unwrap_or_default(),
            self_uid: response_head.to_uid.to_owned().unwrap_or_default(),
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

    pub(crate) fn parse_fake_chain(
        body: PushMsgBody,
        ctx: &Context,
    ) -> Result<MessageChain, String> {
        let make_group_extra = |body: &PushMsgBody| -> Option<GroupMessageUniqueElem> {
            Some(GroupMessageUniqueElem {
                group_uin: body
                    .response_head
                    .as_ref()?
                    .grp
                    .as_ref()?
                    .group_code
                    .unwrap_or_default() as u32,
                group_member_info: Some(dda!(BotGroupMember {
                    member_card: Some(
                        body.response_head
                            .as_ref()?
                            .grp
                            .as_ref()?
                            .group_card
                            .clone()?
                    ),
                    member_name: Some(
                        body.response_head
                            .as_ref()?
                            .grp
                            .as_ref()?
                            .group_card
                            .clone()?
                    ),
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
        let mut chain = MessagePacker::parse_chain(body, ctx)?;
        chain.typ = typ;
        Ok(chain)
    }

    pub(crate) fn parse_private_file(
        body: PushMsgBody,
        ctx: &Context,
    ) -> Result<MessageChain, String> {
        let msg_content = body
            .body
            .as_ref()
            .and_then(|b| b.msg_content.clone())
            .ok_or_else(|| "missing msg_content".to_string())?;

        let mut base_chain = MessagePacker::parse_chain(body, ctx)?;
        let extra = FileExtra::decode(Bytes::from(msg_content))
            .map_err(|e| format!("failed to decode FileExtra: {:?}", e))?;
        let file = extra
            .file
            .as_ref()
            .ok_or_else(|| "missing file".to_string())?;
        if let Some(file_size) = &file.file_size
            && let Some(file_name) = &file.file_name
            && let Some(file_md5) = &file.file_md5
            && let Some(file_uuid) = &file.file_uuid
            && let Some(file_hash) = &file.file_hash
        {
            base_chain.entities.push(Entity::File(dda!(FileEntity {
                file_size: *file_size as u64,
                file_name: file_name.to_owned(),
                file_md5: Bytes::from(file_md5.to_owned()),
                extra: Some(FileUnique::C2C(FileC2CUnique {
                    file_uuid: Some(file_uuid.to_owned()),
                    file_hash: Some(file_hash.to_owned()),
                })),
            })));
        } else {
            return Err("missing file fields".to_string());
        }
        Ok(base_chain)
    }
}
