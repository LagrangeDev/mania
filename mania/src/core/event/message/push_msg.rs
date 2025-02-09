use crate::core::event::notify::group_sys_request_join::GroupSysRequestJoinEvent;
use crate::core::event::prelude::*;
use crate::core::protos::message::{GroupJoin, PushMsg};
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

impl ClientEvent for PushMessageEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        todo!()
    }

    fn parse(bytes: Bytes, _: &Context) -> CEParseResult {
        let packet = PushMsg::decode(bytes)?;
        let typ = packet
            .message
            .as_ref()
            .and_then(|msg| msg.content_head.as_ref())
            .map(|content_head| content_head.r#type)
            .ok_or_else(|| EventError::OtherError("Cannot get typ in PushMsg".to_string()))?;
        let packet_type =
            PkgType::try_from(typ).map_err(|_| EventError::UnknownOlpushMessageTypeError(typ))?;
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
                if let Some(msg_content) = packet
                    .message
                    .and_then(|content| content.body)
                    .and_then(|body| body.msg_content)
                {
                    let join = GroupJoin::decode(Bytes::from(msg_content))?;
                    extra
                        .as_mut()
                        .unwrap()
                        .push(Box::new(GroupSysRequestJoinEvent {
                            target_uid: join.target_uid,
                            group_uin: join.group_uin,
                        }));
                }
            }
            // TODO: handle other message types
            _ => {
                tracing::warn!("receive unknown message type: {:?}", packet_type);
            }
        }
        Ok(ClientResult::with_extra(Box::new(Self { chain }), extra))
    }
}
