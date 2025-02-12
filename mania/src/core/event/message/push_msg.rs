use crate::core::entity::group_sys_enum::{
    GroupMemberDecreaseEventType, GroupMemberIncreaseEventType,
};
use crate::core::event::notify::friend_sys_poke::FriendSysPokeEvent;
use crate::core::event::notify::friend_sys_recall::FriendSysRecallEvent;
use crate::core::event::notify::group_sys_decrease::GroupSysDecreaseEvent;
use crate::core::event::notify::group_sys_increase::GroupSysIncreaseEvent;
use crate::core::event::notify::group_sys_poke::GroupSysPokeEvent;
use crate::core::event::notify::group_sys_reaction::GroupSysReactionEvent;
use crate::core::event::notify::group_sys_recall::GroupSysRecallEvent;
use crate::core::event::notify::group_sys_request_join::GroupSysRequestJoinEvent;
use crate::core::event::prelude::*;
use crate::core::protos::message::{
    FriendRecall, GeneralGrayTipInfo, GroupChange, GroupJoin, NotifyMessageBody, OperatorInfo,
    PushMsg,
};
use crate::message::chain::MessageChain;
use crate::message::packer::MessagePacker;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
enum PkgType {
    PrivateMessage = 166,
    GroupMessage = 82,
    TempMessage = 141,
    Event0x210 = 0x210, // friend related event (528)
    Event0x2DC = 0x2DC, // group related event (732)
    PrivateRecordMessage = 208,
    PrivateFileMessage = 529,
    GroupRequestInvitationNotice = 525, // from group member invitation
    GroupRequestJoinNotice = 84,        // directly entered
    GroupInviteNotice = 87,             // the bot self is being invited
    GroupAdminChangedNotice = 44,       // admin change, both on and off
    GroupMemberIncreaseNotice = 33,
    GroupMemberDecreaseNotice = 34,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
enum Event0x2DCSubType {
    GroupMuteNotice = 12,
    SubType16 = 16,
    GroupRecallNotice = 17,
    GroupEssenceNotice = 21,
    GroupGreyTipNotice = 20,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
#[allow(clippy::enum_variant_names)]
enum Event0x2DCSubType16Field13 {
    GroupMemberSpecialTitleNotice = 6,
    GroupNameChangeNotice = 12,
    GroupTodoNotice = 23,
    GroupReactionNotice = 35,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
enum Event0x210SubType {
    FriendRequestNotice = 35,
    GroupMemberEnterNotice = 38,
    FriendDeleteOrPinChangedNotice = 39,
    FriendRecallNotice = 138,
    ServicePinChanged = 199,
    FriendPokeNotice = 290,
    GroupKickNotice = 212,
}

#[command("trpc.msg.olpush.OlPushService.MsgPush")]
#[derive(Debug, ServerEvent)]
pub struct PushMessageEvent {
    pub chain: Option<MessageChain>,
}

fn extract_msg_content(packet: &mut PushMsg, err_tip: &str) -> Result<Bytes, EventError> {
    packet
        .message
        .as_mut()
        .and_then(|content| content.body.as_mut())
        .and_then(|body| body.msg_content.take())
        .map(Bytes::from)
        .ok_or_else(|| EventError::OtherError(err_tip.to_string()))
}

impl ClientEvent for PushMessageEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        todo!()
    }

    fn parse(bytes: Bytes, _: &Context) -> CEParseResult {
        let mut packet = PushMsg::decode(bytes)?;
        let typ = packet
            .message
            .as_ref()
            .and_then(|msg| msg.content_head.as_ref())
            .map(|content_head| content_head.r#type)
            .ok_or_else(|| EventError::OtherError("Cannot get typ in PushMsg".to_string()))?;
        let packet_type = PkgType::try_from(typ).map_err(|_| {
            EventError::OtherError(format!("receive unknown olpush message type: {:?}", typ))
        })?;
        let mut chain: Option<MessageChain> = None;
        let mut extra: Option<Vec<Box<dyn ServerEvent>>> = match packet_type {
            PkgType::PrivateMessage
            | PkgType::GroupMessage
            | PkgType::TempMessage
            | PkgType::PrivateRecordMessage => None,
            _ => Some(Vec::with_capacity(1)),
        };
        match packet_type {
            PkgType::PrivateMessage
            | PkgType::GroupMessage
            | PkgType::TempMessage
            | PkgType::PrivateRecordMessage => {
                chain = Some(
                    MessagePacker::parse_chain(packet.message.ok_or_else(|| {
                        EventError::OtherError("PushMsgBody is None".to_string())
                    })?)
                    .map_err(|e| EventError::OtherError(format!("parse_chain failed: {}", e)))?,
                );
            }
            PkgType::PrivateFileMessage => {
                chain = Some(
                    MessagePacker::parse_private_file(packet.message.ok_or_else(|| {
                        EventError::OtherError("PushMsgBody is None".to_string())
                    })?)
                    .map_err(|e| {
                        EventError::OtherError(format!("parse_file_chain failed: {}", e))
                    })?,
                )
            }
            PkgType::GroupRequestJoinNotice => {
                let msg_content =
                    extract_msg_content(&mut packet, "GroupRequestJoinNotice missing msg_content")?;
                let join = GroupJoin::decode(msg_content)?;
                extra
                    .as_mut()
                    .unwrap()
                    .push(Box::new(GroupSysRequestJoinEvent {
                        target_uid: join.target_uid,
                        group_uin: join.group_uin,
                    }));
            }
            PkgType::GroupMemberIncreaseNotice => {
                let msg_content = extract_msg_content(
                    &mut packet,
                    "GroupMemberIncreaseNotice missing msg_content",
                )?;
                let increase = GroupChange::decode(msg_content)?;
                let invitor_uid = increase
                    .operator
                    .map(String::from_utf8)
                    .transpose()
                    .map_err(|e| {
                        EventError::OtherError(format!(
                            "Failed to parse invitor_uid in GroupChange: {}",
                            e
                        ))
                    })?;
                extra
                    .as_mut()
                    .unwrap()
                    .push(Box::new(GroupSysIncreaseEvent {
                        group_uin: increase.group_uin,
                        member_uid: increase.member_uid,
                        invitor_uid,
                        event_type: GroupMemberIncreaseEventType::try_from(increase.decrease_type)
                            .unwrap_or_default(),
                    }));
            }
            PkgType::GroupMemberDecreaseNotice => {
                let msg_content = extract_msg_content(
                    &mut packet,
                    "GroupMemberDecreaseNotice missing msg_content",
                )?;
                let decrease = GroupChange::decode(msg_content)?;
                match decrease.decrease_type {
                    3 => {
                        // bot itself is kicked
                        let op = OperatorInfo::decode(Bytes::from(decrease.operator.ok_or(
                            EventError::OtherError(
                                "Cannot get operator in GroupChange".to_string(),
                            ),
                        )?))?;
                        extra
                            .as_mut()
                            .unwrap()
                            .push(Box::new(GroupSysDecreaseEvent {
                                group_uin: decrease.group_uin,
                                member_uid: decrease.member_uid,
                                operator_uid: op.operator_field1.map(|o| o.operator_uid),
                                event_type: GroupMemberDecreaseEventType::try_from(
                                    decrease.decrease_type,
                                )
                                .unwrap_or_default(),
                            }));
                    }
                    _ => {
                        let op_uid = decrease
                            .operator
                            .and_then(|operator| String::from_utf8(operator).ok());
                        extra
                            .as_mut()
                            .unwrap()
                            .push(Box::new(GroupSysDecreaseEvent {
                                group_uin: decrease.group_uin,
                                member_uid: decrease.member_uid,
                                operator_uid: op_uid,
                                event_type: GroupMemberDecreaseEventType::try_from(
                                    decrease.decrease_type,
                                )
                                .unwrap_or_default(),
                            }));
                    }
                }
            }
            PkgType::Event0x2DC => {
                extra = process_event_0x2dc(&mut packet, &mut extra)?.take();
            }
            PkgType::Event0x210 => {
                extra = process_event_0x210(&mut packet, &mut extra)?.take();
            }
            // TODO: handle other message types
            _ => {
                tracing::warn!("receive unknown message type: {:?}", packet_type);
            }
        }
        Ok(ClientResult::with_extra(Box::new(Self { chain }), extra))
    }
}

fn extract_0x2dc_fucking_head<T>(msg_content: Bytes) -> Result<(u32, T), EventError>
where
    T: prost::Message + Default,
{
    let mut packet_reader = PacketReader::new(msg_content);
    let group_uin = packet_reader.u32();
    packet_reader.u8();
    let proto =
        packet_reader.read_with_length::<_, { PREFIX_U16 | PREFIX_LENGTH_ONLY }>(|p| p.bytes());
    let msg_body = T::decode(proto)?;
    Ok((group_uin, msg_body))
}

fn extract_0x_sub_type(packet: &PushMsg) -> Result<u32, EventError> {
    packet
        .message
        .as_ref()
        .and_then(|msg| msg.content_head.as_ref())
        .and_then(|content_head| content_head.sub_type)
        .ok_or_else(|| EventError::OtherError("Cannot get sub_type in PushMsg".to_string()))
}

struct PokeArgs {
    action: String,
    operator_uin: u32,
    target_uin: u32,
    suffix: String,
    action_img_url: String,
}

fn extract_poke_info(gt: &mut GeneralGrayTipInfo) -> PokeArgs {
    let mut templates: HashMap<String, String> = gt
        .msg_templ_param
        .drain(..)
        .map(|param| (param.key, param.value))
        .collect();
    let action = templates
        .remove("action_str")
        .or_else(|| templates.remove("alt_str1"))
        .unwrap_or_default();
    let operator_uin = templates
        .get("uin_str1")
        .unwrap_or(&"".to_string())
        .parse::<u32>()
        .unwrap_or_default();
    let target_uin = templates
        .get("uin_str2")
        .unwrap_or(&"".to_string())
        .parse::<u32>()
        .unwrap_or_default();
    let suffix = templates.remove("suffix").unwrap_or_default();
    let action_img_url = templates.remove("action_img_url").unwrap_or_default();
    PokeArgs {
        action,
        operator_uin,
        target_uin,
        suffix,
        action_img_url,
    }
}

#[allow(clippy::single_match)] // FIXME:
fn process_event_0x2dc<'a>(
    packet: &'a mut PushMsg,
    extra: &'a mut Option<Vec<Box<dyn ServerEvent>>>,
) -> Result<&'a mut Option<Vec<Box<dyn ServerEvent>>>, EventError> {
    let sub_type = Event0x2DCSubType::try_from(extract_0x_sub_type(packet)?).map_err(|err| {
        EventError::OtherError(format!(
            "receive unknown olpush message 0x2dc sub type: {:?}",
            err
        ))
    })?;
    match sub_type {
        Event0x2DCSubType::SubType16 => {
            let msg_content = extract_msg_content(packet, "0x2dc SubType16 missing msg_content")?;
            let (group_uin, msg_body) =
                extract_0x2dc_fucking_head::<NotifyMessageBody>(msg_content)?;
            let ev = Event0x2DCSubType16Field13::try_from(msg_body.field13.unwrap_or_default())
                .map_err(|e| {
                    EventError::OtherError(format!(
                        "Failed to parse 0x2dc sub type 16 field 13: {}",
                        e
                    ))
                })?;
            match ev {
                Event0x2DCSubType16Field13::GroupReactionNotice => {
                    let data_2 = msg_body
                        .reaction
                        .as_ref()
                        .and_then(|d| d.data.to_owned())
                        .and_then(|d| d.data)
                        .ok_or_else(|| {
                            EventError::OtherError(
                                "Missing reaction data_2 in 0x2dc sub type 16 field 13".into(),
                            )
                        })?;
                    let data_3 = data_2.data.as_ref().ok_or(EventError::OtherError(
                        "Missing reaction data_3 in 0x2dc sub type 16 field 13".into(),
                    ))?;
                    extra.as_mut().unwrap().push(Box::new(GroupSysReactionEvent {
                        target_group_uin: group_uin,
                        target_sequence: data_2.target.ok_or_else(
                            || EventError::OtherError("Missing target_sequence in reaction in 0x2dc sub type 16 field 13".into())
                        )?.sequence,
                        operator_uid: data_3.operator_uid.to_owned(),
                        is_add: data_3.r#type == 1,
                        code: data_3.code.to_owned(),
                        count: data_3.count,
                    }));
                }
                _ => {}
            }
        }
        Event0x2DCSubType::GroupRecallNotice => {
            let msg_content =
                extract_msg_content(packet, "0x2dc GroupRecallNotice missing msg_content")?;
            let (_, recall_notify) = extract_0x2dc_fucking_head::<NotifyMessageBody>(msg_content)?;
            let recall = recall_notify.recall.ok_or(EventError::OtherError(
                "Missing recall meta in 0x2dc sub type 17".into(),
            ))?;
            let tip_info = recall.tip_info.unwrap_or_default();
            let meta = recall
                .recall_messages
                .first()
                .ok_or(EventError::OtherError(
                    "Missing recall message in 0x2dc sub type 17".into(),
                ))?;
            extra.as_mut().unwrap().push(Box::new(GroupSysRecallEvent {
                group_uin: recall_notify.group_uin,
                author_uid: meta.author_uid.to_owned(),
                operator_uid: recall.operator_uid,
                sequence: meta.sequence as u32,
                time: meta.time,
                random: meta.random,
                tip: tip_info.tip,
            }));
        }
        Event0x2DCSubType::GroupGreyTipNotice => {
            let msg_content = extract_msg_content(packet, "0x2dc sub type 20 missing msg_content")?;
            let (group_uin, mut grey_tip) =
                extract_0x2dc_fucking_head::<NotifyMessageBody>(msg_content)?;
            let gray_tip_info = match grey_tip.gray_tip_info.as_mut() {
                Some(info) if info.busi_type == 12 => info,
                _ => return Ok(extra),
            };
            let poke_args = extract_poke_info(gray_tip_info);
            extra.as_mut().unwrap().push(Box::new(GroupSysPokeEvent {
                group_uin,
                operator_uin: poke_args.operator_uin,
                target_uin: poke_args.target_uin,
                action: poke_args.action,
                suffix: poke_args.suffix,
                action_img_url: poke_args.action_img_url,
            }));
        }
        _ => {
            tracing::warn!("TODO: unhandled 0x2dc sub type: {:?}", sub_type);
        }
    }
    Ok(extra)
}

#[allow(clippy::single_match)] // FIXME:
fn process_event_0x210<'a>(
    packet: &'a mut PushMsg,
    extra: &'a mut Option<Vec<Box<dyn ServerEvent>>>,
) -> Result<&'a mut Option<Vec<Box<dyn ServerEvent>>>, EventError> {
    let sub_type = Event0x210SubType::try_from(extract_0x_sub_type(packet)?).map_err(|err| {
        EventError::OtherError(format!(
            "receive unknown olpush message 0x210 sub type: {:?}",
            err
        ))
    })?;
    match sub_type {
        Event0x210SubType::FriendRecallNotice => {
            let msg_content = packet
                .message
                .as_ref()
                .and_then(|m| m.body.as_ref())
                .and_then(|b| b.msg_content.as_ref())
                .ok_or_else(|| {
                    EventError::OtherError(
                        "Missing msg_content in Event0x210SubType::FriendRecallNotice".into(),
                    )
                })?;
            let response_head = packet
                .message
                .as_ref()
                .and_then(|m| m.response_head.as_ref())
                .ok_or_else(|| {
                    EventError::OtherError(
                        "Missing response_head in Event0x210SubType::FriendRecallNotice".into(),
                    )
                })?;
            let friend_request = FriendRecall::decode(Bytes::from(msg_content.to_owned()))?;
            let info = friend_request.info.ok_or(EventError::OtherError(
                "Missing friend request info in 0x210 sub type 138".into(),
            ))?;
            extra.as_mut().unwrap().push(Box::new(FriendSysRecallEvent {
                from_uid: response_head.from_uid.to_owned().unwrap_or_default(),
                client_sequence: info.sequence,
                time: info.time,
                random: info.random,
                tip: info.tip_info.unwrap_or_default().tip.unwrap_or_default(),
            }));
        }
        Event0x210SubType::FriendPokeNotice => {
            let msg_content =
                extract_msg_content(packet, "0x210 FriendPokeNotice missing msg_content")?;
            let mut grey_tip = GeneralGrayTipInfo::decode(msg_content)?;
            if grey_tip.busi_type != 12 {
                return Ok(extra);
            }
            let poke_args = extract_poke_info(&mut grey_tip);
            extra.as_mut().unwrap().push(Box::new(FriendSysPokeEvent {
                operator_uin: poke_args.operator_uin,
                target_uin: poke_args.target_uin,
                action: poke_args.action,
                suffix: poke_args.suffix,
                action_img_url: poke_args.action_img_url,
            }));
        }
        _ => {
            tracing::warn!("TODO: unhandled 0x210 sub type: {:?}", sub_type);
        }
    }
    Ok(extra)
}
