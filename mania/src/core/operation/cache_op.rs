use crate::core::business::BusinessHandle;
use crate::core::event::downcast_mut_major_event;
use crate::core::event::system::fetch_friend::FetchFriendsEvent;
use crate::core::event::system::fetch_members::FetchMembersEvent;
use crate::entity::bot_friend::{BotFriend, BotFriendGroup};
use crate::entity::bot_group_member::BotGroupMember;
use crate::{ManiaResult, dda};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

impl BusinessHandle {
    pub async fn resolve_uid(
        self: &Arc<Self>,
        group_uin: Option<u32>,
        friend_uin: u32,
    ) -> ManiaResult<String> {
        if self.cache.uin2uid.is_empty() {
            self.resolve_friends_uid_and_friend_groups().await?;
        }
        if let Some(uin) = group_uin
            && !self.cache.cached_group_members.contains_key(&uin)
        {
            self.resolve_members_uid(uin).await?;
        }
        self.cache
            .uin2uid
            .get(&friend_uin)
            .map(|uid_ref| uid_ref.value().clone())
            .ok_or_else(|| crate::ManiaError::GenericError(Cow::from("Uin not found")))
    }

    async fn resolve_friends_uid_and_friend_groups(self: &Arc<Self>) -> ManiaResult<()> {
        let mut next_uin: Option<u32> = None;
        let mut friends: Vec<BotFriend> = Vec::new();
        let mut friend_groups: HashMap<u32, String> = HashMap::new();
        loop {
            let mut event = dda!(FetchFriendsEvent { next_uin });
            let mut result = self.send_event(&mut event).await?;
            let event: &mut FetchFriendsEvent = downcast_mut_major_event(&mut result)
                .ok_or_else(|| crate::ManiaError::GenericError("Downcast error".into()))?;
            match event.next_uin {
                Some(uin) => {
                    friend_groups.extend(event.friend_groups.to_owned());
                    for friend in event.friends.iter_mut() {
                        let group_id = friend
                            .group
                            .as_ref()
                            .ok_or_else(|| {
                                crate::ManiaError::GenericError(Cow::from("Missing group id"))
                            })?
                            .group_id;
                        if let Some(name) = friend_groups.get(&group_id) {
                            friend.group = Some(BotFriendGroup {
                                group_id,
                                group_name: name.to_owned(),
                            });
                        }
                    }
                    friends.extend(event.friends.to_owned());
                    next_uin = Some(uin);
                }
                None => break,
            }
        }
        friends.iter().for_each(|bf| {
            self.cache.uin2uid.insert(bf.uin, bf.uid.clone());
        });
        self.cache.cached_friends.write().await.clear();
        self.cache.cached_friends.write().await.extend(friends);
        Ok(())
    }

    async fn resolve_members_uid(self: &Arc<Self>, group_uin: u32) -> ManiaResult<()> {
        let mut group_members: Vec<BotGroupMember> = Vec::new();
        let mut token: Option<String> = None;
        loop {
            let mut event = dda!(FetchMembersEvent { group_uin, token });
            let mut result = self.send_event(&mut event).await?;
            let event: &mut FetchMembersEvent = downcast_mut_major_event(&mut result)
                .ok_or_else(|| crate::ManiaError::GenericError("Downcast error".into()))?;
            match event.token.as_ref() {
                Some(t) => {
                    group_members.extend(event.group_members.to_owned());
                    token = Some(t.to_owned());
                }
                None => break,
            }
        }
        group_members.iter().for_each(|bgm| {
            self.cache.uin2uid.insert(bgm.uin, bgm.uid.clone());
        });
        self.cache
            .cached_group_members
            .insert(group_uin, group_members);
        Ok(())
    }
}
