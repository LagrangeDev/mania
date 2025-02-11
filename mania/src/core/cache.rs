use crate::entity::bot_friend::BotFriend;
use crate::entity::bot_group_member::BotGroupMember;
use dashmap::DashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    Full,
    Half,
    None,
}

pub struct Cache {
    pub(crate) cache_mode: CacheMode,
    pub(crate) uin2uid: Option<DashMap<u32, String>>,
    pub(crate) uid2uin: Option<DashMap<String, u32>>,
    pub(crate) cached_friends: Option<DashMap<u32, BotFriend>>,
    pub(crate) cached_group_members: Option<DashMap<u32, Vec<BotGroupMember>>>,
}

impl Cache {
    pub fn new(cache_mode: CacheMode) -> Self {
        match cache_mode {
            CacheMode::Full => Self::full(),
            CacheMode::Half => Self::half(),
            CacheMode::None => Self::none(),
        }
    }

    fn full() -> Self {
        Self {
            cache_mode: CacheMode::Full,
            uin2uid: Some(DashMap::new()),
            uid2uin: Some(DashMap::new()),
            cached_friends: Some(DashMap::new()),
            cached_group_members: Some(DashMap::new()),
        }
    }

    fn half() -> Self {
        Self {
            cache_mode: CacheMode::Half,
            uin2uid: None,
            uid2uin: None,
            cached_friends: Some(DashMap::new()),
            cached_group_members: Some(DashMap::new()),
        }
    }

    fn none() -> Self {
        Self {
            cache_mode: CacheMode::None,
            uin2uid: None,
            uid2uin: None,
            cached_friends: None,
            cached_group_members: None,
        }
    }

    pub(crate) fn insert_uin_uid(&self, uin: u32, uid: String) {
        // SAFETY: we can ensure that the DashMap is not None
        self.uin2uid.as_ref().unwrap().insert(uin, uid.clone());
        self.uid2uin.as_ref().unwrap().insert(uid, uin);
    }
}
