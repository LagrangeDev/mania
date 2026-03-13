use dashmap::DashMap;
use mania_core::CacheMode;
use mania_core::entity::bot_friend::BotFriend;
use mania_core::entity::bot_group_member::BotGroupMember;

pub struct Cache {
    pub cache_mode: CacheMode,
    pub uin2uid: Option<DashMap<u32, String>>,
    pub uid2uin: Option<DashMap<String, u32>>,
    pub cached_friends: Option<DashMap<u32, BotFriend>>,
    pub cached_group_members: Option<DashMap<u32, Vec<BotGroupMember>>>,
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

    pub fn insert_uin_uid(&self, uin: u32, uid: String) {
        self.uin2uid
            .as_ref()
            .map(|uin2uid| uin2uid.insert(uin, uid.clone()));
        self.uid2uin
            .as_ref()
            .map(|uid2uin| uid2uin.insert(uid, uin));
    }
}
