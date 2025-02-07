use crate::entity::bot_friend::BotFriend;
use crate::entity::bot_group_member::BotGroupMember;
use dashmap::DashMap;
use tokio::sync::RwLock;

pub struct Cache {
    pub uin2uid: DashMap<u32, String>,
    pub cached_friends: RwLock<Vec<BotFriend>>,
    pub cached_group_members: DashMap<u32, Vec<BotGroupMember>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            uin2uid: DashMap::new(),
            cached_friends: RwLock::new(Vec::new()),
            cached_group_members: DashMap::new(),
        }
    }
}
