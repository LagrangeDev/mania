use crate::core::entity::group_sys_enum::{
    GroupEssenceSetFlag, GroupMemberDecreaseEventType, GroupMemberIncreaseEventType,
};
use crate::core::event::notify::bot_sys_rename::BotSysRenameEvent;
use crate::core::event::notify::friend_sys_new::FriendSysNewEvent;
use crate::core::event::notify::friend_sys_poke::FriendSysPokeEvent;
use crate::core::event::notify::friend_sys_recall::FriendSysRecallEvent;
use crate::core::event::notify::friend_sys_rename::FriendSysRenameEvent;
use crate::core::event::notify::friend_sys_request::FriendSysRequestEvent;
use crate::core::event::notify::group_sys_admin::GroupSysAdminEvent;
use crate::core::event::notify::group_sys_decrease::GroupSysDecreaseEvent;
use crate::core::event::notify::group_sys_essence::GroupSysEssenceEvent;
use crate::core::event::notify::group_sys_increase::GroupSysIncreaseEvent;
use crate::core::event::notify::group_sys_invite::GroupSysInviteEvent;
use crate::core::event::notify::group_sys_member_enter::GroupSysMemberEnterEvent;
use crate::core::event::notify::group_sys_member_mute::GroupSysMemberMuteEvent;
use crate::core::event::notify::group_sys_mute::GroupSysMuteEvent;
use crate::core::event::notify::group_sys_name_change::GroupSysNameChangeEvent;
use crate::core::event::notify::group_sys_pin_change::GroupSysPinChangeEvent;
use crate::core::event::notify::group_sys_poke::GroupSysPokeEvent;
use crate::core::event::notify::group_sys_reaction::GroupSysReactionEvent;
use crate::core::event::notify::group_sys_recall::GroupSysRecallEvent;
use crate::core::event::notify::group_sys_request_invitation::GroupSysRequestInvitationEvent;
use crate::core::event::notify::group_sys_request_join::GroupSysRequestJoinEvent;
use crate::core::event::notify::group_sys_special_title::GroupSysSpecialTitleEvent;
use crate::core::event::notify::group_sys_todo::GroupSysTodoEvent;
use crate::core::event::prelude::*;
use crate::core::protos::message::{
    Event0x210Sub39Notify, FriendRecall, FriendRequest, GeneralGrayTipInfo, GroupAdmin,
    GroupChange, GroupInvitation, GroupInvite, GroupJoin, GroupMemberEnterNotify, GroupMute,
    GroupNameChange, NewFriend, NotifyMessageBody, OperatorInfo, PushMsg, SelfRenameNotify,
    SpecialTittleNotify,
};
use crate::message::chain::MessageChain;
use crate::message::packer::MessagePacker;
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;

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
    SelfRenameNotice = 29,
    FriendRequestNotice = 35,
    GroupMemberEnterNotice = 38,
    FriendDeleteOrPinChangedNotice = 39,
    FriendRecallNotice = 138,
    SubType179 = 179,
    ServicePinChanged = 199,
    SubType226 = 226,
    FriendPokeNotice = 290,
    GroupKickNotice = 212,
}

#[command("trpc.msg.olpush.OlPushService.MsgPush")]
#[derive(Debug, ServerEvent)]
pub struct PushMessageEvent {
    pub chain: Option<MessageChain>,
}

fn extract_msg_body_content(packet: &mut PushMsg) -> Option<Bytes> {
    packet
        .message
        .as_mut()
        .and_then(|content| content.body.as_mut())
        .and_then(|body| body.msg_content.take())
        .map(Bytes::from)
}

fn extract_stable_msg_content(packet: &mut PushMsg, err_tip: &str) -> Result<Bytes, EventError> {
    packet
        .message
        .as_mut()
        .and_then(|content| content.body.as_mut())
        .and_then(|body| body.msg_content.take())
        .map(Bytes::from)
        .ok_or_else(|| EventError::OtherError(err_tip.to_string()))
}

fn extract_unstable_msg_content(packet: &mut PushMsg, err_tip: &str) -> Result<Bytes, EventError> {
    packet
        .message
        .as_mut()
        .and_then(|content| content.body.as_mut())
        .and_then(|body| body.msg_content.take())
        .map(Bytes::from)
        .ok_or_else(|| EventError::InternalWarning(err_tip.to_string()))
}

impl ClientEvent for PushMessageEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        todo!()
    }

    fn parse(bytes: Bytes, ctx: &Context) -> CEParseResult {
        let mut packet = PushMsg::decode(bytes)?;
        let typ = packet
            .message
            .as_ref()
            .and_then(|msg| msg.content_head.as_ref())
            .map(|content_head| content_head.r#type)
            .ok_or_else(|| EventError::OtherError("Cannot get typ in PushMsg".to_string()))?;
        let packet_type = PkgType::try_from(typ).map_err(|_| {
            EventError::InternalWarning(format!("receive unknown olpush message type: {:?}", typ))
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
                    MessagePacker::parse_chain(
                        packet.message.ok_or_else(|| {
                            EventError::OtherError("PushMsgBody is None".to_string())
                        })?,
                        ctx,
                    )
                    .map_err(|e| EventError::OtherError(format!("parse_chain failed: {}", e)))?,
                );
            }
            PkgType::PrivateFileMessage => {
                chain = Some(
                    MessagePacker::parse_private_file(
                        packet.message.ok_or_else(|| {
                            EventError::OtherError("PushMsgBody is None".to_string())
                        })?,
                        ctx,
                    )
                    .map_err(|e| {
                        EventError::OtherError(format!("parse_file_chain failed: {}", e))
                    })?,
                )
            }
            PkgType::GroupRequestInvitationNotice => {
                let msg_content = extract_stable_msg_content(
                    &mut packet,
                    "GroupRequestInvitationNotice missing msg_content",
                )?;
                let invite = GroupInvitation::decode(msg_content)?;
                match invite.cmd {
                    87 => {
                        let info_inner = invite
                            .info
                            .ok_or_else(|| {
                                EventError::OtherError("GroupInvitation missing data".to_string())
                            })?
                            .inner
                            .ok_or_else(|| {
                                EventError::OtherError("GroupInvitation missing inner".to_string())
                            })?;
                        extra
                            .as_mut()
                            .unwrap()
                            .push(Box::new(GroupSysRequestInvitationEvent {
                                group_uin: info_inner.group_uin,
                                target_uid: info_inner.target_uid,
                                invitor_uid: info_inner.invitor_uid,
                            }));
                    }
                    _ => {
                        Err(EventError::InternalWarning(
                            "GroupRequestInvitationNotice unknown cmd".to_string(),
                        ))?;
                    }
                }
            }
            PkgType::GroupRequestJoinNotice => {
                let msg_content = extract_stable_msg_content(
                    &mut packet,
                    "GroupRequestJoinNotice missing msg_content",
                )?;
                let join = GroupJoin::decode(msg_content)?;
                extra
                    .as_mut()
                    .unwrap()
                    .push(Box::new(GroupSysRequestJoinEvent {
                        target_uid: join.target_uid,
                        group_uin: join.group_uin,
                    }));
            }
            PkgType::GroupInviteNotice => {
                let msg_content = extract_stable_msg_content(
                    &mut packet,
                    "GroupInviteNotice missing msg_content",
                )?;
                let invite = GroupInvite::decode(msg_content)?;
                extra.as_mut().unwrap().push(Box::new(GroupSysInviteEvent {
                    group_uin: invite.group_uin,
                    invitor_uid: invite.invitor_uid,
                }));
            }
            PkgType::GroupAdminChangedNotice => {
                let msg_content = extract_stable_msg_content(
                    &mut packet,
                    "GroupAdminChangedNotice missing msg_content",
                )?;
                let mut change = GroupAdmin::decode(msg_content)?;
                let body = change
                    .body
                    .take()
                    .ok_or_else(|| EventError::OtherError("GroupAdmin missing body".to_string()))?;
                let (enabled, uid) = body
                    .extra_enable
                    .map(|extra| (true, extra.admin_uid))
                    .or_else(|| body.extra_disable.map(|extra| (false, extra.admin_uid)))
                    .ok_or_else(|| {
                        EventError::OtherError("GroupAdmin missing extra".to_string())
                    })?;
                extra.as_mut().unwrap().push(Box::new(GroupSysAdminEvent {
                    group_uin: change.group_uin,
                    uid,
                    is_promoted: enabled,
                }));
            }
            PkgType::GroupMemberIncreaseNotice => {
                let msg_content = extract_stable_msg_content(
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
                let msg_content = extract_stable_msg_content(
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
                extra = process_event_0x2dc(ctx, &mut packet, &mut extra)?.take();
            }
            PkgType::Event0x210 => {
                extra = process_event_0x210(ctx, &mut packet, &mut extra)?.take();
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

#[derive(Deserialize, Debug)]
struct SpecialTitleUserInfo {
    // cmd: u8,
    data: String,
    text: String,
    #[serde(flatten)]
    ex: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct SpecialTitleMedalInfo {
    // cmd: u8,
    data: String,
    text: String,
    url: String,
    #[serde(flatten)]
    ex: HashMap<String, serde_json::Value>,
}

fn process_event_0x2dc<'a>(
    _: &Context,
    packet: &'a mut PushMsg,
    extra: &'a mut Option<Vec<Box<dyn ServerEvent>>>,
) -> Result<&'a mut Option<Vec<Box<dyn ServerEvent>>>, EventError> {
    let sub_type = Event0x2DCSubType::try_from(extract_0x_sub_type(packet)?).map_err(|err| {
        EventError::InternalWarning(format!(
            "receive unknown olpush message 0x2dc sub type: {:?}",
            err
        ))
    })?;
    match sub_type {
        Event0x2DCSubType::GroupMuteNotice => {
            let msg_content =
                extract_stable_msg_content(packet, "0x2dc GroupMuteNotice missing msg_content")?;
            let mute = GroupMute::decode(msg_content)?;
            let state = mute
                .data
                .ok_or_else(|| EventError::OtherError("GroupMute missing data".to_string()))?
                .state
                .ok_or_else(|| EventError::OtherError("GroupMute missing state".to_string()))?;
            if state.target_uid.is_none() {
                extra.as_mut().unwrap().push(Box::new(GroupSysMuteEvent {
                    group_uin: mute.group_uin,
                    operator_uid: mute.operator_uid,
                    is_muted: state.duration != 0,
                }));
            } else {
                extra
                    .as_mut()
                    .unwrap()
                    .push(Box::new(GroupSysMemberMuteEvent {
                        group_uin: mute.group_uin,
                        operator_uid: mute.operator_uid,
                        target_uid: state.target_uid.ok_or_else(|| {
                            EventError::OtherError("Missing target_uid".to_string())
                        })?,
                        duration: state.duration,
                    }));
            }
        }
        Event0x2DCSubType::SubType16 => {
            let msg_content =
                extract_unstable_msg_content(packet, "0x2dc SubType16 missing msg_content")?;
            let (group_uin, msg_body) =
                extract_0x2dc_fucking_head::<NotifyMessageBody>(msg_content)?;
            let ev = Event0x2DCSubType16Field13::try_from(msg_body.field13.unwrap_or_default())
                .map_err(|e| {
                    EventError::InternalWarning(format!(
                        "Failed to parse 0x2dc sub type 16 field 13: {}",
                        e
                    ))
                })?;
            match ev {
                Event0x2DCSubType16Field13::GroupMemberSpecialTitleNotice => {
                    let content = SpecialTittleNotify::decode(Bytes::from(msg_body.event_param))?;
                    let re = Regex::new(r#"<(.*?)>"#)
                        .map_err(|_| EventError::OtherError("Failed to compile regex".into()))?;
                    let captures: Vec<_> = re.captures_iter(&content.notify_inner).collect();
                    if captures.len() == 2 {
                        let user_info_json = &captures[0][1];
                        let medal_info_json = &captures[1][1];
                        let user_info: SpecialTitleUserInfo = serde_json::from_str(user_info_json)
                            .map_err(|_| {
                                EventError::OtherError("Failed to parse special title".into())
                            })?;
                        let medal_info: SpecialTitleMedalInfo =
                            serde_json::from_str(medal_info_json).map_err(|_| {
                                EventError::OtherError("Failed to parse special title".into())
                            })?;
                        extra
                            .as_mut()
                            .unwrap()
                            .push(Box::new(GroupSysSpecialTitleEvent {
                                target_uin: content.target_uin,
                                target_nickname: user_info.text,
                                special_title: medal_info.text,
                                special_title_detail_url: medal_info.url,
                                group_uin,
                            }));
                    } else {
                        Err(EventError::OtherError(
                            "Failed to parse special title".into(),
                        ))?;
                    }
                }
                Event0x2DCSubType16Field13::GroupNameChangeNotice => {
                    let param = GroupNameChange::decode(Bytes::from(msg_body.event_param))?;
                    extra
                        .as_mut()
                        .unwrap()
                        .push(Box::new(GroupSysNameChangeEvent {
                            group_uin,
                            name: param.name,
                        }));
                }
                Event0x2DCSubType16Field13::GroupTodoNotice => {
                    extra.as_mut().unwrap().push(Box::new(GroupSysTodoEvent {
                        group_uin,
                        operator_uid: msg_body.operator_uid.to_owned(),
                    }));
                }
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
            }
        }
        Event0x2DCSubType::GroupRecallNotice => {
            let msg_content =
                extract_stable_msg_content(packet, "0x2dc GroupRecallNotice missing msg_content")?;
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
        Event0x2DCSubType::GroupEssenceNotice => {
            let msg_content =
                extract_stable_msg_content(packet, "0x2dc GroupEssenceNotice missing msg_content")?;
            let (group_uin, mut essence) =
                extract_0x2dc_fucking_head::<NotifyMessageBody>(msg_content)?;
            let essence_msg = essence.essence_message.take().ok_or_else(|| {
                EventError::OtherError("Missing essence_message in 0x2dc sub type 21".into())
            })?;
            extra.as_mut().unwrap().push(Box::new(GroupSysEssenceEvent {
                group_uin,
                sequence: essence_msg.msg_sequence,
                random: essence_msg.random,
                set_flag: GroupEssenceSetFlag::try_from(essence_msg.set_flag).unwrap_or_default(),
                from_uin: essence_msg.author_uin,
                operator_uin: essence_msg.operator_uin,
            }));
        }
        Event0x2DCSubType::GroupGreyTipNotice => {
            let msg_content =
                extract_unstable_msg_content(packet, "0x2dc sub type 20 missing msg_content")?;
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
    }
    Ok(extra)
}

fn process_event_0x210<'a>(
    ctx: &Context,
    packet: &'a mut PushMsg,
    extra: &'a mut Option<Vec<Box<dyn ServerEvent>>>,
) -> Result<&'a mut Option<Vec<Box<dyn ServerEvent>>>, EventError> {
    let sub_type = Event0x210SubType::try_from(extract_0x_sub_type(packet)?).map_err(|err| {
        EventError::InternalWarning(format!(
            "receive unknown olpush message 0x210 sub type: {:?}",
            err
        ))
    })?;
    match sub_type {
        Event0x210SubType::SelfRenameNotice => {
            let msg_content =
                extract_stable_msg_content(packet, "0x210 SelfRenameNotice missing msg_content")?;
            let rename_data = SelfRenameNotify::decode(msg_content)?
                .body
                .ok_or_else(|| EventError::OtherError("Missing body in 0x210 sub type 29".into()))?
                .rename_data
                .ok_or_else(|| {
                    EventError::OtherError("Missing rename_data in 0x210 sub type 29".into())
                })?;
            ctx.key_store
                .info
                .load()
                .name
                .store(Arc::new(rename_data.nick_name.clone()));
            extra.as_mut().unwrap().push(Box::new(BotSysRenameEvent {
                nickname: rename_data.nick_name,
            }));
        }
        Event0x210SubType::FriendRequestNotice => {
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
            let info = FriendRequest::decode(Bytes::from(msg_content.to_owned()))?
                .info
                .ok_or_else(|| {
                    EventError::OtherError(
                        "Missing friend request info in 0x210 sub type 35".into(),
                    )
                })?;
            extra
                .as_mut()
                .unwrap()
                .push(Box::new(FriendSysRequestEvent {
                    source_uin: response_head.from_uin,
                    source_uid: info.source_uid,
                    message: info.message,
                    source: info.source,
                }));
        }
        Event0x210SubType::GroupMemberEnterNotice => {
            let msg_content = extract_unstable_msg_content(
                packet,
                "0x210 GroupMemberEnterNotice missing msg_content",
            )?;
            let info = GroupMemberEnterNotify::decode(msg_content)?;
            let detail = info
                .body
                .ok_or(EventError::InternalWarning(
                    "Missing body in 0x210 sub type 38".into(),
                ))?
                .info
                .ok_or(EventError::InternalWarning(
                    "Missing info in 0x210 sub type 38".into(),
                ))?
                .detail
                .ok_or(EventError::InternalWarning(
                    "Missing detail in 0x210 sub type 38".into(),
                ))?;
            let style = detail.style.ok_or(EventError::InternalWarning(
                "Missing style in 0x210 sub type 38".into(),
            ))?;
            extra
                .as_mut()
                .unwrap()
                .push(Box::new(GroupSysMemberEnterEvent {
                    group_uin: detail.group_id,
                    group_member_uin: detail.group_member_uin,
                    style_id: style.style_id,
                }));
        }
        Event0x210SubType::FriendDeleteOrPinChangedNotice => {
            let msg_content = extract_stable_msg_content(
                packet,
                "0x210 FriendDeleteOrPinChangedNotice missing msg_content",
            )?;
            let mut nt = Event0x210Sub39Notify::decode(msg_content)?;
            let nt_body = nt.body.take().ok_or_else(|| {
                EventError::OtherError("Missing body in 0x210 sub type 39".into())
            })?;
            match nt_body.r#type {
                7 => match nt_body.pin_changed {
                    Some(pc) => {
                        let mut body = pc.body.ok_or_else(|| {
                            EventError::OtherError("Missing pin_changed body".into())
                        })?;
                        extra
                            .as_mut()
                            .unwrap()
                            .push(Box::new(GroupSysPinChangeEvent {
                                uid: body.uid,
                                group_uin: body.group_uin,
                                is_pin: body
                                    .info
                                    .take()
                                    .is_some_and(|info| !info.timestamp.is_empty()),
                            }));
                    }
                    None => {
                        Err(EventError::OtherError(
                            "Missing pin_changed in 0x210 sub type 39 type 7".into(),
                        ))?;
                    }
                },
                20 => match nt_body.data {
                    Some(data) => {
                        let body = data.rename_data.ok_or_else(|| {
                            EventError::OtherError("Missing data.rename_data".into())
                        })?;
                        extra.as_mut().unwrap().push(Box::new(FriendSysRenameEvent {
                            uid: data.uid,
                            nickname: body.nick_name,
                        }));
                    }
                    None => {
                        Err(EventError::OtherError(
                            "Missing data in 0x210 sub type 39 type 20".into(),
                        ))?;
                    }
                },
                _ => {
                    Err(EventError::OtherError(
                        "Unknown 0x210 sub type 39 type".into(),
                    ))?;
                }
            }
        }
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
                extract_stable_msg_content(packet, "0x210 FriendPokeNotice missing msg_content")?;
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
        Event0x210SubType::SubType179 | Event0x210SubType::SubType226 => {
            let msg_content =
                extract_unstable_msg_content(packet, "0x210 SubType179 missing msg_content")?;
            let new_friend = NewFriend::decode(msg_content)?.info.ok_or_else(|| {
                EventError::OtherError("Missing info in 0x210 sub type 179".into())
            })?;
            extra.as_mut().unwrap().push(Box::new(FriendSysNewEvent {
                from_uid: new_friend.uid,
                from_nickname: new_friend.nick_name,
                msg: new_friend.message,
            }));
        }
        Event0x210SubType::ServicePinChanged | Event0x210SubType::GroupKickNotice => {
            Err(EventError::InternalWarning(format!(
                "TODO: handle 0x210 sub type {:?}",
                sub_type
            )))?;
        }
    }
    Ok(extra)
}
