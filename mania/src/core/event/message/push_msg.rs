use crate::core::event::prelude::*;
use crate::core::protos::message::PushMsg;
use crate::message::chain::MessageChain;
use crate::message::packer::MessagePacker;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
enum PkgType {
    PrivateMessage = 166,
    GroupMessage = 82,
    TempMessage = 141,
    Event0x210 = 528, // friend related event
    Event0x2DC = 732, // group related event
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

#[commend("trpc.msg.olpush.OlPushService.MsgPush")]
#[derive(Debug, ServerEvent)]
pub struct PushMessageEvent {
    pub chain: Option<MessageChain>,
}

impl ClientEvent for PushMessageEvent {
    fn build(&self, _: &Context) -> BinaryPacket {
        todo!()
    }

    fn parse(bytes: Bytes, _: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        let packet = PushMsg::decode(bytes)?;
        let typ = packet
            .message
            .as_ref()
            .and_then(|msg| msg.content_head.as_ref())
            .map(|content_head| content_head.r#type)
            .unwrap();
        let packet_type =
            PkgType::try_from(typ).map_err(|_| EventError::UnknownOlpushMessageTypeError(typ))?;
        let mut chain = MessageChain::default(); // FIXME: maybe exist better way to handle this
        match packet_type {
            PkgType::PrivateMessage | PkgType::GroupMessage | PkgType::TempMessage => {
                chain = MessagePacker::parse_chain(packet.message.expect("PushMsgBody is None"))
                    .map_err(|e| EventError::OtherError(format!("parse_chain failed: {}", e)))?;
            }
            PkgType::PrivateFileMessage => {
                chain =
                    MessagePacker::parse_private_file(packet.message.expect("PushMsgBody is None"))
                        .map_err(|e| {
                            EventError::OtherError(format!("parse_private_file failed: {}", e))
                        })?;
            }
            // TODO: handle other message types
            _ => {
                tracing::warn!("receive unknown message type: {:?}", packet_type);
            }
        }
        Ok(Box::new(PushMessageEvent { chain: Some(chain) }))
    }
}
