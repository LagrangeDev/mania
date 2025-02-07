use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub member_name: String,
    pub special_title: Option<String>,
    pub join_time: DateTime<Utc>,
    pub last_msg_time: DateTime<Utc>,
    pub shut_up_timestamp: DateTime<Utc>,
}

impl BotGroupMember {
    pub fn avatar(&self) -> String {
        format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=640", self.uin)
    }
}
