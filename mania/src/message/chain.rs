use crate::core::protos::message::Elem;
use crate::dda;
use crate::entity::bot_friend::BotFriend;
use crate::entity::bot_group_member::BotGroupMember;
use crate::message::entity::Entity;
use crate::utility::random_gen::RandomGenerator;
use chrono::{DateTime, Utc};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Debug, PartialEq, Eq)]
enum MessageTag {
    Group,
    Friend,
    Temp,
}

#[derive(Debug, Default)]
pub enum MessageType {
    Friend(FriendMessageUniqueElem),
    Group(GroupMessageUniqueElem),
    Temp,
    #[default]
    None,
}

#[derive(Debug, Default)]
pub struct GroupMessageUniqueElem {
    pub group_uin: u32,
    pub group_member_info: Option<BotGroupMember>,
}

#[derive(Debug, Default)]
pub struct FriendMessageUniqueElem {
    pub friend_info: Option<BotFriend>,
    pub client_sequence: ClientSequence,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct MessageId(pub u64);

impl Default for MessageId {
    fn default() -> Self {
        Self(((0x01000000u32 as u64) << 32) | ClientSequence::default().0 as u64)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClientSequence(pub u32);

impl Default for ClientSequence {
    fn default() -> Self {
        Self(RandomGenerator::random_num(100000000, u32::MAX))
    }
}

#[derive(Default)]
pub struct MessageChain {
    pub typ: MessageType,
    pub(crate) uid: String,
    pub(crate) self_uid: String,
    pub target_uin: u32,
    pub friend_uin: u32,
    pub message_id: MessageId,
    pub time: DateTime<Utc>,
    pub sequence: u32,
    pub(crate) elements: Vec<Elem>,
    pub entities: Vec<Entity>,
}

/// For debugging output, console, and log display
///
/// aka ToPreviewString() in [Lagrange.Core](https://github.com/LagrangeDev/Lagrange.Core)
impl Debug for MessageChain {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let header = match &self.typ {
            MessageType::Group(group_elem) => format!(
                "[MessageChain({} -> {})] ",
                self.friend_uin, group_elem.group_uin
            ),
            MessageType::Friend(_) | MessageType::Temp => {
                format!("[MessageChain({})] ", self.friend_uin)
            }
            MessageType::None => "[MessageChain(Empty)] ".to_string(),
        };
        let entities_preview = self
            .entities
            .iter()
            .map(|entity| format!("{:?}", entity))
            .collect::<Vec<String>>()
            .join(" | ");
        write!(f, "{}{}", header, entities_preview)
    }
}

/// For previewing messages that are actually sent, **such as the outer preview for Forward Message**
///
/// aka ToPreviewText() in [Lagrange.Core](https://github.com/LagrangeDev/Lagrange.Core)
impl Display for MessageChain {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for entity in &self.entities {
            write!(f, "{}", entity)?;
        }
        Ok(())
    }
}

impl MessageChain {
    pub fn is_group(&self) -> bool {
        matches!(self.typ, MessageType::Group(_))
    }

    pub(crate) fn friend(friend_uin: u32, friend_uid: &str, self_uid: &str) -> Self {
        dda!(Self {
            typ: MessageType::Friend(FriendMessageUniqueElem::default()),
            self_uid: self_uid.to_string(),
            uid: friend_uid.to_string(),
            friend_uin,
        })
    }

    pub(crate) fn group(group_uin: u32) -> Self {
        dda!(Self {
            typ: MessageType::Group(dda!(GroupMessageUniqueElem { group_uin })),
        })
    }
}
