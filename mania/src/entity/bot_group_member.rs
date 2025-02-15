use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchGroupMemberStrategy {
    Simple,
    Full,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum GroupMemberPermission {
    #[default]
    Member = 0,
    Owner = 1,
    Admin = 2,
}

impl TryFrom<u32> for GroupMemberPermission {
    type Error = ();

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(GroupMemberPermission::Member),
            1 => Ok(GroupMemberPermission::Owner),
            2 => Ok(GroupMemberPermission::Admin),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BotGroupMember {
    pub uin: u32,
    pub uid: String,
    pub permission: GroupMemberPermission,
    pub group_level: u32,
    pub member_card: Option<String>,
    pub special_title: Option<String>,
    /// Below fields only available when `FetchGroupMemberStrategy::Full` is set
    pub member_name: Option<String>,
    pub join_time: Option<DateTime<Utc>>,
    pub last_msg_time: Option<DateTime<Utc>>,
    pub shut_up_timestamp: Option<DateTime<Utc>>,
}

impl BotGroupMember {
    pub fn avatar(&self) -> String {
        format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=640", self.uin)
    }
}
