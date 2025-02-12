use crate::core::business::BusinessHandle;
use crate::core::cache::CacheMode;
use crate::core::event::downcast_mut_major_event;
use crate::core::event::system::fetch_friend::FetchFriendsEvent;
use crate::core::event::system::fetch_members::FetchMembersEvent;
use crate::entity::bot_friend::{BotFriend, BotFriendGroup};
use crate::entity::bot_group_member::BotGroupMember;
use crate::{ManiaError, ManiaResult, dda};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

impl BusinessHandle {
    pub async fn uin2uid(
        self: &Arc<Self>,
        uin: u32,
        group_uin: Option<u32>,
    ) -> ManiaResult<String> {
        match self.cache.cache_mode {
            CacheMode::Full | CacheMode::Half => {
                if self.cache.cache_mode == CacheMode::Full
                    && self.cache.uin2uid.as_ref().unwrap().is_empty()
                {
                    self.refresh_friends_cache().await?;
                }
                if let Some(group_uin) = group_uin
                    && !self
                        .cache
                        .cached_group_members
                        .as_ref()
                        .unwrap()
                        .contains_key(&group_uin)
                {
                    self.refresh_group_members_cache(group_uin).await?;
                }
                self.resolve_uin2uid_within_cache(uin, group_uin).await
            }
            CacheMode::None => {
                if let Some(group_uin) = group_uin {
                    self.fast_fetch_group_members_uid(uin, group_uin).await
                } else {
                    self.fast_fetch_friends_uid(uin).await
                }
            }
        }
    }

    pub async fn uid2uin(self: &Arc<Self>, uid: &str, group_uin: Option<u32>) -> ManiaResult<u32> {
        match self.cache.cache_mode {
            CacheMode::Full | CacheMode::Half => {
                if self.cache.cache_mode == CacheMode::Full
                    && self.cache.uid2uin.as_ref().unwrap().is_empty()
                {
                    self.refresh_friends_cache().await?;
                }
                if let Some(group_uin) = group_uin
                    && !self
                        .cache
                        .cached_group_members
                        .as_ref()
                        .unwrap()
                        .contains_key(&group_uin)
                {
                    self.refresh_group_members_cache(group_uin).await?;
                }
                self.resolve_uid2uin_within_cache(uid, group_uin).await
            }
            CacheMode::None => {
                if let Some(group_uin) = group_uin {
                    self.fast_fetch_group_members_uin(uid, group_uin).await
                } else {
                    self.fast_fetch_friends_uin(uid).await
                }
            }
        }
    }

    async fn resolve_uin2uid_within_cache(
        self: &Arc<Self>,
        uin: u32,
        group_uin: Option<u32>,
    ) -> ManiaResult<String> {
        if let Some(uid) = self.cache.uin2uid.as_ref().and_then(|m| m.get(&uin)) {
            return Ok(uid.value().to_string());
        }
        if let Some(group_id) = group_uin {
            let group_members = self
                .cache
                .cached_group_members
                .as_ref()
                .unwrap()
                .get(&group_id)
                .ok_or_else(|| ManiaError::GenericError(Cow::from("Group not found")))?;
            group_members
                .iter()
                .find(|member| member.uin == uin)
                .map(|member| member.uid.clone())
                .ok_or_else(|| ManiaError::GenericError(Cow::from("Member not found")))
        } else {
            let friend = self
                .cache
                .cached_friends
                .as_ref()
                .unwrap()
                .get(&uin)
                .ok_or_else(|| ManiaError::GenericError(Cow::from("Friend not found")))?;
            Ok(friend.value().uid.clone())
        }
    }

    async fn resolve_uid2uin_within_cache(
        self: &Arc<Self>,
        uid: &str,
        group_uin: Option<u32>,
    ) -> ManiaResult<u32> {
        if let Some(uin) = self.cache.uid2uin.as_ref().and_then(|m| m.get(uid)) {
            return Ok(*uin.value());
        }
        if let Some(group_id) = group_uin {
            let group_members = self
                .cache
                .cached_group_members
                .as_ref()
                .unwrap()
                .get(&group_id)
                .ok_or_else(|| ManiaError::GenericError(Cow::from("Group not found")))?;
            group_members
                .iter()
                .find(|member| member.uid == uid)
                .map(|member| member.uin)
                .ok_or_else(|| ManiaError::GenericError(Cow::from("Member not found")))
        } else {
            let friend = self
                .cache
                .cached_friends
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.value().uid == uid)
                .ok_or_else(|| ManiaError::GenericError(Cow::from("Friend not found")))?;
            Ok(*friend.key())
        }
    }

    async fn iter_fetch_friends<T, F>(self: &Arc<Self>, mut process: F) -> ManiaResult<Option<T>>
    where
        F: FnMut(&mut FetchFriendsEvent) -> ManiaResult<Option<T>>,
    {
        let mut next_uin: Option<u32> = None;
        loop {
            let mut event = dda!(FetchFriendsEvent { next_uin });
            let mut result = self.send_event(&mut event).await?;
            let event: &mut FetchFriendsEvent = downcast_mut_major_event(&mut result)
                .ok_or_else(|| ManiaError::GenericError("Downcast error".into()))?;
            if let Some(val) = process(event)? {
                return Ok(Some(val));
            }
            if let Some(n) = event.next_uin {
                next_uin = Some(n);
            } else {
                break;
            }
        }
        Ok(None::<T>)
    }

    pub(crate) async fn refresh_friends_cache(self: &Arc<Self>) -> ManiaResult<()> {
        if self.cache.cache_mode == CacheMode::None {
            tracing::warn!("Cache mode is None, no need to refresh friends cache");
            return Ok(());
        }
        let mut friends: HashMap<u32, BotFriend> = HashMap::new();
        let mut friend_groups: HashMap<u32, String> = HashMap::new();
        self.iter_fetch_friends(|event: &mut FetchFriendsEvent| {
            friend_groups.extend(event.friend_groups.to_owned());
            for friend in event.friends.iter_mut() {
                let group_id = friend
                    .group
                    .as_ref()
                    .ok_or_else(|| ManiaError::GenericError(Cow::from("Missing group id")))?
                    .group_id;
                if let Some(name) = friend_groups.get(&group_id) {
                    friend.group = Some(BotFriendGroup {
                        group_id,
                        group_name: name.clone(),
                    });
                }
                friends.insert(friend.uin, friend.to_owned());
                if self.cache.cache_mode == CacheMode::Full {
                    self.cache.insert_uin_uid(friend.uin, friend.uid.clone());
                }
            }
            Ok(None::<()>)
        })
        .await?;
        let cached_friends = self.cache.cached_friends.as_ref().unwrap();
        cached_friends.clear();
        for (uin, friend) in friends.iter() {
            cached_friends.insert(*uin, friend.to_owned());
        }
        Ok(())
    }

    async fn fast_fetch_friends_uid(self: &Arc<Self>, uin: u32) -> ManiaResult<String> {
        let res = self
            .iter_fetch_friends(|event| {
                Ok(event
                    .friends
                    .iter()
                    .find(|f| f.uin == uin)
                    .map(|f| f.uid.clone()))
            })
            .await?;
        res.ok_or_else(|| ManiaError::GenericError(Cow::from("Friend not found")))
    }

    async fn fast_fetch_friends_uin(self: &Arc<Self>, uid: &str) -> ManiaResult<u32> {
        let res = self
            .iter_fetch_friends(|event| {
                Ok(event.friends.iter().find(|f| f.uid == uid).map(|f| f.uin))
            })
            .await?;
        res.ok_or_else(|| ManiaError::GenericError(Cow::from("Friend not found")))
    }
    async fn iter_fetch_group<T, F>(
        self: &Arc<Self>,
        group_uin: u32,
        mut process: F,
    ) -> ManiaResult<Option<T>>
    where
        F: FnMut(&mut FetchMembersEvent) -> ManiaResult<Option<T>>,
    {
        let mut token: Option<String> = None;
        loop {
            let mut event = dda!(FetchMembersEvent { group_uin, token });
            let mut result = self.send_event(&mut event).await?;
            let event: &mut FetchMembersEvent = downcast_mut_major_event(&mut result)
                .ok_or_else(|| ManiaError::GenericError("Downcast error".into()))?;
            if let Some(val) = process(event)? {
                return Ok(Some(val));
            }
            if let Some(t) = event.token.as_ref() {
                token = Some(t.to_owned());
            } else {
                break;
            }
        }
        Ok(None::<T>)
    }

    pub(crate) async fn refresh_group_members_cache(
        self: &Arc<Self>,
        group_uin: u32,
    ) -> ManiaResult<()> {
        if self.cache.cache_mode == CacheMode::None {
            tracing::warn!("Cache mode is None, no need to refresh group members cache");
            return Ok(());
        }
        let mut group_members: Vec<BotGroupMember> = Vec::new();
        self.iter_fetch_group(group_uin, |event| {
            group_members.extend(event.group_members.clone());
            Ok(None::<()>)
        })
        .await?;
        if self.cache.cache_mode == CacheMode::Full {
            group_members.iter().for_each(|bgm| {
                self.cache.insert_uin_uid(bgm.uin, bgm.uid.clone());
            });
        }
        self.cache
            .cached_group_members
            .as_ref()
            .unwrap()
            .insert(group_uin, group_members);
        Ok(())
    }

    async fn fast_fetch_group_members_uid(
        self: &Arc<Self>,
        uin: u32,
        group_uin: u32,
    ) -> ManiaResult<String> {
        let res = self
            .iter_fetch_group(group_uin, |event| {
                Ok(event
                    .group_members
                    .iter()
                    .find(|m| m.uin == uin)
                    .map(|m| m.uid.clone()))
            })
            .await?;
        res.ok_or_else(|| ManiaError::GenericError(Cow::from("Member not found")))
    }

    async fn fast_fetch_group_members_uin(
        self: &Arc<Self>,
        uid: &str,
        group_uin: u32,
    ) -> ManiaResult<u32> {
        let res = self
            .iter_fetch_group(group_uin, |event| {
                Ok(event
                    .group_members
                    .iter()
                    .find(|m| m.uid == uid)
                    .map(|m| m.uin))
            })
            .await?;
        res.ok_or_else(|| ManiaError::GenericError(Cow::from("Member not found")))
    }
}
