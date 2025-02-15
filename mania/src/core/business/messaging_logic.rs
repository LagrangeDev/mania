use crate::core::business::LogicRegistry;
use crate::core::business::{BusinessHandle, LogicFlow};
use crate::core::event::message::push_msg::PushMessageEvent;
use crate::core::event::message::send_message::SendMessageEvent;
use crate::core::event::notify::bot_sys_rename::BotSysRenameEvent;
use crate::core::event::notify::friend_sys_new::FriendSysNewEvent;
use crate::core::event::notify::friend_sys_poke::FriendSysPokeEvent;
use crate::core::event::notify::friend_sys_recall::FriendSysRecallEvent;
use crate::core::event::notify::friend_sys_rename::FriendSysRenameEvent;
use crate::core::event::notify::friend_sys_request::FriendSysRequestEvent;
use crate::core::event::notify::group_sys_admin::GroupSysAdminEvent;
use crate::core::event::notify::group_sys_decrease::GroupSysDecreaseEvent;
use crate::core::event::notify::group_sys_essence::GroupSysEssenceEvent;
use crate::core::event::notify::group_sys_increase::GroupSysIncreaseEvent;
use crate::core::event::notify::group_sys_invite::GroupSysInviteEvent;
use crate::core::event::notify::group_sys_member_enter::GroupSysMemberEnterEvent;
use crate::core::event::notify::group_sys_member_mute::GroupSysMemberMuteEvent;
use crate::core::event::notify::group_sys_mute::GroupSysMuteEvent;
use crate::core::event::notify::group_sys_name_change::GroupSysNameChangeEvent;
use crate::core::event::notify::group_sys_pin_change::GroupSysPinChangeEvent;
use crate::core::event::notify::group_sys_poke::GroupSysPokeEvent;
use crate::core::event::notify::group_sys_reaction::GroupSysReactionEvent;
use crate::core::event::notify::group_sys_recall::GroupSysRecallEvent;
use crate::core::event::notify::group_sys_request_invitation::GroupSysRequestInvitationEvent;
use crate::core::event::notify::group_sys_request_join::GroupSysRequestJoinEvent;
use crate::core::event::notify::group_sys_special_title::GroupSysSpecialTitleEvent;
use crate::core::event::notify::group_sys_todo::GroupSysTodoEvent;
use crate::core::event::prelude::*;
use crate::entity::bot_group_member::FetchGroupMemberStrategy;
use crate::event::friend::friend_poke::FriendPokeEvent;
use crate::event::friend::{
    FriendEvent, friend_message, friend_new, friend_recall, friend_rename, friend_request,
};
use crate::event::group::group_pin_changed::ChatType;
use crate::event::group::group_poke::GroupPokeEvent;
use crate::event::group::group_reaction::GroupReactionEvent;
use crate::event::group::group_recall::GroupRecallEvent;
use crate::event::group::{
    GroupEvent, group_admin_changed, group_essence, group_invitation, group_invitation_request,
    group_join_request, group_member_decrease, group_member_enter, group_member_increase,
    group_member_mute, group_message, group_mute, group_name_change, group_pin_changed,
    group_special_title, group_todo,
};
use crate::event::system::{SystemEvent, bot_rename, temp_message};
use crate::message::chain::{MessageChain, MessageType};
use crate::message::entity::Entity;
use crate::message::entity::file::FileUnique;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(
    SendMessageEvent,
    PushMessageEvent,
    GroupSysRequestJoinEvent,
    GroupSysInviteEvent,
    GroupSysAdminEvent,
    GroupSysPokeEvent,
    GroupSysReactionEvent,
    GroupSysRecallEvent,
    GroupSysEssenceEvent,
    GroupSysIncreaseEvent,
    GroupSysDecreaseEvent,
    GroupSysMuteEvent,
    GroupSysMemberMuteEvent,
    GroupSysNameChangeEvent,
    GroupSysTodoEvent,
    GroupSysSpecialTitleEvent,
    GroupSysMemberEnterEvent,
    GroupSysPinChangeEvent,
    GroupSysRequestInvitationEvent,
    FriendSysRecallEvent,
    FriendSysPokeEvent,
    FriendSysNewEvent,
    FriendSysRenameEvent,
    FriendSysRequestEvent,
    BotSysRenameEvent
)]
async fn messaging_logic(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
    flow: LogicFlow,
) -> &dyn ServerEvent {
    tracing::trace!("[{}] Handling event: {:?}", flow, event);
    match flow {
        LogicFlow::InComing => messaging_logic_incoming(event, handle).await,
        LogicFlow::OutGoing => messaging_logic_outgoing(event, handle).await,
    }
}

// FIXME: avoid take things from event
// FIXME: (TODO) make it return Result(?)
async fn messaging_logic_incoming(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    {
        if let Some(msg) = event.as_any_mut().downcast_mut::<PushMessageEvent>() {
            if let Some(mut chain) = msg.chain.take() {
                resolve_incoming_chain(&mut chain, handle.clone()).await;
                resolve_chain_metadata(&mut chain, handle.clone()).await;
                // TODO: MessageFilter.Filter(push.Chain);
                // TODO?: sb tx! Collection.Invoker.PostEvent(new GroupInvitationEvent(groupUin, chain.FriendUin, sequence));
                match &chain.typ {
                    MessageType::Group(_) => {
                        if let Err(e) =
                            handle
                                .event_dispatcher
                                .group
                                .send(Some(GroupEvent::GroupMessage(
                                    group_message::GroupMessageEvent { chain },
                                )))
                        {
                            tracing::error!("Failed to send group_message event: {:?}", e);
                        }
                    }
                    MessageType::Friend(_) => {
                        if let Err(e) = handle.event_dispatcher.friend.send(Some(
                            FriendEvent::FriendMessageEvent(friend_message::FriendMessageEvent {
                                chain,
                            }),
                        )) {
                            tracing::error!("Failed to send friend_message event: {:?}", e);
                        }
                    }
                    MessageType::Temp => {
                        if let Err(e) = handle.event_dispatcher.system.send(Some(
                            SystemEvent::TempMessageEvent(temp_message::TempMessageEvent { chain }),
                        )) {
                            tracing::error!("Failed to send temp_message event: {:?}", e);
                        }
                    }
                    _ => {}
                }
            } else {
                tracing::trace!("Empty message chain in PushMessageEvent");
            }
            return event;
        }
    }
    {
        if let Some(req) = event
            .as_any_mut()
            .downcast_mut::<GroupSysRequestJoinEvent>()
        {
            let target_uin = match handle.resolve_stranger_uid2uin(&req.target_uid).await {
                Ok(uin) => uin,
                Err(e) => {
                    tracing::error!(
                        "Failed to resolve stranger uid for {}: {:?}",
                        req.target_uid,
                        e
                    );
                    return event;
                }
            };
            let requests = match handle.fetch_group_requests().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to fetch group requests: {:?}", e);
                    return event;
                }
            };
            if let Some(r) = requests
                .iter()
                .find(|r| r.group_uin == req.group_uin && r.target_member_uin == target_uin)
            {
                if let Err(e) =
                    handle
                        .event_dispatcher
                        .group
                        .send(Some(GroupEvent::GroupJoinRequest(
                            group_join_request::GroupJoinRequestEvent {
                                group_uin: req.group_uin,
                                target_uin,
                                target_nickname: r.target_member_card.to_owned(),
                                invitor_uin: r.invitor_member_uin.unwrap_or_default(),
                                answer: r.comment.to_owned().unwrap_or_default(),
                                request_seq: r.sequence,
                            },
                        )))
                {
                    tracing::error!("Failed to send group join request event: {:?}", e);
                }
            } else {
                tracing::warn!("No group join request found for target: {}", target_uin);
            }
            return event;
        }

        if let Some(invite) = event.as_any_mut().downcast_mut::<GroupSysInviteEvent>() {
            let invitor_uin = handle
                .resolve_stranger_uid2uin_fast(&invite.invitor_uid)
                .await;
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupInvitation(
                    group_invitation::GroupInvitationEvent {
                        group_uin: invite.group_uin,
                        invitor_uin,
                        sequence: None,
                    },
                )))
            {
                tracing::error!("Failed to send group invitation event: {:?}", e);
            }
            return event;
        }

        if let Some(change) = event.as_any_mut().downcast_mut::<GroupSysAdminEvent>() {
            let group_uin = change.group_uin;
            let admin_uin = handle.uid2uin_fast(&change.uid, Some(group_uin)).await;
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupAdminChanged(
                    group_admin_changed::GroupAdminChangedEvent {
                        group_uin,
                        admin_uin,
                        is_promote: change.is_promoted,
                    },
                )))
            {
                tracing::error!("Failed to send group admin change event: {:?}", e);
            }
            return event;
        }

        if let Some(poke) = event.as_any_mut().downcast_mut::<GroupSysPokeEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupPoke(GroupPokeEvent {
                    group_uin: poke.group_uin,
                    operator_uin: poke.operator_uin,
                    target_uin: poke.target_uin,
                    action: poke.action.to_owned(),
                    suffix: poke.suffix.to_owned(),
                    action_img_url: poke.action_img_url.to_owned(),
                })))
            {
                tracing::error!("Failed to send group poke event: {:?}", e);
            }
            return event;
        }

        if let Some(join) = event.as_any_mut().downcast_mut::<GroupSysIncreaseEvent>() {
            let member_uin = handle.resolve_stranger_uid2uin_fast(&join.member_uid).await;
            let invitor_uin = handle
                .uid2uin(
                    join.invitor_uid.as_deref().unwrap_or(""),
                    Some(join.group_uin),
                )
                .await
                .map(Some)
                .unwrap_or_else(|e| {
                    tracing::error!("uid2uin error: {:?}", e);
                    None
                });
            if let Err(e) =
                handle
                    .event_dispatcher
                    .group
                    .send(Some(GroupEvent::GroupMemberIncrease(
                        group_member_increase::GroupMemberIncreaseEvent {
                            group_uin: join.group_uin,
                            member_uin,
                            invitor_uin,
                            event_type: std::mem::take(&mut join.event_type),
                        },
                    )))
            {
                tracing::error!("Failed to send group increase event: {:?}", e);
            }
            return event;
        }

        if let Some(leave) = event.as_any_mut().downcast_mut::<GroupSysDecreaseEvent>() {
            let member_uin = handle
                .resolve_stranger_uid2uin_fast(&leave.member_uid)
                .await;
            let operator_uin = handle
                .uid2uin(
                    leave.operator_uid.as_deref().unwrap_or(""),
                    Some(leave.group_uin),
                )
                .await
                .map(Some)
                .unwrap_or_else(|e| {
                    tracing::error!("uid2uin error: {:?}", e);
                    None
                });
            if let Err(e) =
                handle
                    .event_dispatcher
                    .group
                    .send(Some(GroupEvent::GroupMemberDecrease(
                        group_member_decrease::GroupMemberDecreaseEvent {
                            group_uin: leave.group_uin,
                            member_uin,
                            operator_uin,
                            event_type: std::mem::take(&mut leave.event_type),
                        },
                    )))
            {
                tracing::error!("Failed to send group decrease event: {:?}", e);
            }
            return event;
        }

        if let Some(sys_mute) = event.as_any_mut().downcast_mut::<GroupSysMuteEvent>() {
            let group_uin = sys_mute.group_uin;
            let operator_uin = handle
                .uid2uin_fast(
                    sys_mute.operator_uid.as_deref().unwrap_or(""),
                    Some(group_uin),
                )
                .await;
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupMute(group_mute::GroupMuteEvent {
                    group_uin,
                    operator_uin: Some(operator_uin),
                    is_muted: sys_mute.is_muted,
                })))
            {
                tracing::error!("Failed to send group mute event: {:?}", e);
            }
            return event;
        }

        if let Some(member_mute) = event.as_any_mut().downcast_mut::<GroupSysMemberMuteEvent>() {
            let group_uin = member_mute.group_uin;
            let operator_uin = match &member_mute.operator_uid {
                Some(uid) => Some(handle.uid2uin_fast(uid, Some(group_uin)).await),
                None => None,
            };
            let target_uin = handle
                .resolve_stranger_uid2uin_fast(&member_mute.target_uid)
                .await;
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupMemberMute(
                    group_member_mute::GroupMemberMuteEvent {
                        group_uin,
                        operator_uin,
                        target_uin,
                        duration: member_mute.duration,
                    },
                )))
            {
                tracing::error!("Failed to send group member mute event: {:?}", e);
            }
            return event;
        }

        if let Some(name_change) = event.as_any_mut().downcast_mut::<GroupSysNameChangeEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupNameChange(
                    group_name_change::GroupNameChangeEvent {
                        group_uin: name_change.group_uin,
                        name: name_change.name.to_owned(),
                    },
                )))
            {
                tracing::error!("Failed to send group name change event: {:?}", e);
            }
            return event;
        }

        if let Some(todo) = event.as_any_mut().downcast_mut::<GroupSysTodoEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupTodo(group_todo::GroupTodoEvent {
                    group_uin: todo.group_uin,
                    operator_uin: handle
                        .uid2uin_fast(&todo.operator_uid, Some(todo.group_uin))
                        .await,
                })))
            {
                tracing::error!("Failed to send group todo event: {:?}", e);
            }
            return event;
        }

        if let Some(reaction) = event.as_any_mut().downcast_mut::<GroupSysReactionEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupReaction(GroupReactionEvent {
                    target_group_uin: reaction.target_group_uin,
                    target_sequence: reaction.target_sequence,
                    operator_uin: handle
                        .uid2uin_fast(&reaction.operator_uid, Some(reaction.target_group_uin))
                        .await,
                    is_add: reaction.is_add,
                    code: reaction.code.to_owned(),
                    count: reaction.count,
                })))
            {
                tracing::error!("Failed to send group reaction event: {:?}", e);
            }
            return event;
        }

        if let Some(recall) = event.as_any_mut().downcast_mut::<GroupSysRecallEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupRecall(GroupRecallEvent {
                    group_uin: recall.group_uin,
                    author_uin: handle
                        .uid2uin_fast(&recall.author_uid, Some(recall.group_uin))
                        .await,
                    operator_uin: if let Some(uid) = recall.operator_uid.as_ref() {
                        handle.uid2uin_fast(uid, Some(recall.group_uin)).await
                    } else {
                        0
                    },
                    sequence: recall.sequence,
                    time: recall.time,
                    random: recall.random,
                    tip: recall.tip.to_owned(),
                })))
            {
                tracing::error!("Failed to send group recall event: {:?}", e);
            }
            return event;
        }

        if let Some(essence) = event.as_any_mut().downcast_mut::<GroupSysEssenceEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupEssence(
                    group_essence::GroupEssenceEvent {
                        group_uin: essence.group_uin,
                        sequence: essence.sequence,
                        random: essence.random,
                        is_set: std::mem::take(&mut essence.set_flag),
                        from_uin: essence.from_uin,
                        operator_uin: essence.operator_uin,
                    },
                )))
            {
                tracing::error!("Failed to send group sys essence event: {:?}", e);
            }
            return event;
        }

        if let Some(st) = event
            .as_any_mut()
            .downcast_mut::<GroupSysSpecialTitleEvent>()
        {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupSpecialTitle(
                    group_special_title::GroupSpecialTitleEvent {
                        target_uin: st.target_uin,
                        target_nickname: st.target_nickname.to_owned(),
                        special_title: st.special_title.to_owned(),
                        special_title_detail_url: st.special_title_detail_url.to_owned(),
                        group_uin: st.group_uin,
                    },
                )))
            {
                tracing::error!("Failed to send group special title event: {:?}", e);
            }
            return event;
        }

        if let Some(enter) = event
            .as_any_mut()
            .downcast_mut::<GroupSysMemberEnterEvent>()
        {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupMemberEnter(
                    group_member_enter::GroupMemberEnterEvent {
                        group_uin: enter.group_uin,
                        group_member_uin: enter.group_member_uin,
                        style_id: enter.style_id,
                    },
                )))
            {
                tracing::error!("Failed to send group member enter event: {:?}", e);
            }
            return event;
        }

        if let Some(invite) = event
            .as_any_mut()
            .downcast_mut::<GroupSysRequestInvitationEvent>()
        {
            if let Err(e) =
                handle
                    .event_dispatcher
                    .group
                    .send(Some(GroupEvent::GroupInvitationRequest(
                        group_invitation_request::GroupInvitationRequestEvent {
                            group_uin: invite.group_uin,
                            target_uin: handle
                                .resolve_stranger_uid2uin_fast(&invite.target_uid)
                                .await,
                            invitor_uin: handle
                                .uid2uin_fast(&invite.invitor_uid, Some(invite.group_uin))
                                .await,
                        },
                    )))
            {
                tracing::error!("Failed to send group invitation request event: {:?}", e);
            }
            return event;
        }

        if let Some(recall) = event.as_any_mut().downcast_mut::<FriendSysRecallEvent>() {
            if let Err(e) =
                handle
                    .event_dispatcher
                    .friend
                    .send(Some(FriendEvent::FriendRecallEvent(
                        friend_recall::FriendRecallEvent {
                            friend_uin: handle.uid2uin_fast(&recall.from_uid, None).await,
                            client_sequence: recall.client_sequence,
                            time: recall.time,
                            random: recall.random,
                            tip: recall.tip.to_owned(),
                        },
                    )))
            {
                tracing::error!("Failed to send friend recall event: {:?}", e);
            }
            return event;
        }

        if let Some(pin) = event.as_any_mut().downcast_mut::<GroupSysPinChangeEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupPinChanged(
                    group_pin_changed::PinChangedEvent {
                        chat_type: match pin.group_uin {
                            Some(_) => ChatType::Group,
                            None => ChatType::Friend,
                        },
                        uin: handle.uid2uin_fast(&pin.uid, pin.group_uin).await,
                        is_pin: pin.is_pin,
                    },
                )))
            {
                tracing::error!("Failed to send group pin change event: {:?}", e);
            }
            return event;
        }

        if let Some(poke) = event.as_any_mut().downcast_mut::<FriendSysPokeEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .friend
                .send(Some(FriendEvent::FriendPokeEvent(FriendPokeEvent {
                    operator_uin: poke.operator_uin,
                    target_uin: poke.target_uin,
                    action: poke.action.to_owned(),
                    suffix: poke.suffix.to_owned(),
                    action_url: poke.action_img_url.to_owned(),
                })))
            {
                tracing::error!("Failed to send friend poke event: {:?}", e);
            }
            return event;
        }

        if let Some(friend_new) = event.as_any_mut().downcast_mut::<FriendSysNewEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .friend
                .send(Some(FriendEvent::FriendNewEvent(
                    friend_new::FriendNewEvent {
                        from_uin: handle.uid2uin_fast(&friend_new.from_uid, None).await,
                        from_nickname: friend_new.from_nickname.to_owned(),
                        msg: friend_new.msg.to_owned(),
                    },
                )))
            {
                tracing::error!("Failed to send friend new event: {:?}", e);
            }
            return event;
        }

        if let Some(rename) = event.as_any_mut().downcast_mut::<FriendSysRenameEvent>() {
            if let Err(e) =
                handle
                    .event_dispatcher
                    .friend
                    .send(Some(FriendEvent::FriendRenameEvent(
                        friend_rename::FriendRenameEvent {
                            uin: handle.uid2uin_fast(&rename.uid, None).await,
                            nickname: rename.nickname.to_owned(),
                        },
                    )))
            {
                tracing::error!("Failed to send friend rename event: {:?}", e);
            }
            return event;
        }

        if let Some(request) = event.as_any_mut().downcast_mut::<FriendSysRequestEvent>() {
            if let Err(e) =
                handle
                    .event_dispatcher
                    .friend
                    .send(Some(FriendEvent::FriendRequestEvent(
                        friend_request::FriendRequestEvent {
                            source_uin: handle.uid2uin_fast(&request.source_uid, None).await,
                            message: request.message.to_owned(),
                            source: request.source.to_owned(),
                        },
                    )))
            {
                tracing::error!("Failed to send friend request event: {:?}", e);
            }
            return event;
        }

        if let Some(rename) = event.as_any_mut().downcast_mut::<BotSysRenameEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .system
                .send(Some(SystemEvent::BotRenameEvent(
                    bot_rename::BotRenameEvent {
                        nickname: rename.nickname.to_owned(),
                    },
                )))
            {
                tracing::error!("Failed to send bot rename event: {:?}", e);
            }
            return event;
        }
    }
    event
}

async fn resolve_incoming_chain(chain: &mut MessageChain, handle: Arc<BusinessHandle>) {
    for entity in &mut chain.entities {
        match *entity {
            Entity::Image(ref mut image) => {
                if image.url.contains("&rkey=") {
                    continue;
                }
                let index_node = match image
                    .msg_info
                    .as_ref()
                    .and_then(|info| info.msg_info_body.first())
                    .and_then(|node| node.index.clone())
                {
                    Some(idx) => idx,
                    None => {
                        tracing::warn!("Missing index node in image entity: {:?}", image);
                        continue;
                    }
                };
                let download = match &chain.typ {
                    MessageType::Group(grp) => {
                        handle.download_group_image(grp.group_uin, index_node).await
                    }
                    MessageType::Friend(_) | MessageType::Temp => {
                        handle.download_c2c_image(index_node).await
                    }
                    _ => continue,
                };
                match download {
                    Ok(url) => {
                        image.url = url;
                    }
                    Err(e) => {
                        tracing::error!("Failed to download image: {:?}", e);
                    }
                }
            }
            Entity::MultiMsg(ref mut multi) => {
                // TODO: recursively resolve?
                if multi.chains.is_empty() {
                    let msg = handle
                        .multi_msg_download(chain.uid.clone(), multi.res_id.clone())
                        .await;
                    match msg {
                        Ok(Some(chains)) => {
                            multi.chains = chains;
                        }
                        Ok(None) => {
                            tracing::warn!("No chains found in MultiMsgDownloadEvent");
                        }
                        Err(e) => {
                            tracing::error!("Failed to download MultiMsg: {:?}", e);
                        }
                    }
                }
            }
            Entity::File(ref mut file) => {
                file.file_url = match file.extra.as_ref() {
                    Some(extra) => match extra {
                        FileUnique::Group(grp) => {
                            let group_uin = if let MessageType::Group(ref grp) = chain.typ {
                                grp.group_uin
                            } else {
                                tracing::error!("expected group message, find {:?}", chain.typ);
                                continue;
                            };
                            let file_id = match &grp.file_id {
                                Some(id) => id,
                                None => continue,
                            };
                            handle
                                .download_group_file(group_uin, file_id.clone())
                                .await
                                .ok()
                        }
                        FileUnique::C2C(c2c) => handle
                            .download_c2c_file(
                                c2c.file_uuid.clone(),
                                c2c.file_hash.clone(),
                                Some(chain.uid.clone()),
                            )
                            .await
                            .ok(),
                    },
                    _ => None,
                }
            }
            Entity::Record(ref mut record) => {
                let node = || -> Option<_> {
                    record
                        .msg_info
                        .as_ref()
                        .and_then(|info| info.msg_info_body.first())
                        .and_then(|node| node.index.clone())
                };
                let url_result = match (&record.msg_info, &record.audio_uuid) {
                    (Some(_), _) => match &chain.typ {
                        MessageType::Group(grp) => {
                            handle
                                .download_group_record_by_node(grp.group_uin, node())
                                .await
                        }
                        MessageType::Friend(_) | MessageType::Temp => {
                            handle.download_c2c_record_by_node(node()).await
                        }
                        _ => continue,
                    },
                    (None, Some(uuid)) => match &chain.typ {
                        MessageType::Group(grp) => {
                            handle
                                .download_group_record_by_uuid(grp.group_uin, Some(uuid.clone()))
                                .await
                        }
                        MessageType::Friend(_) | MessageType::Temp => {
                            handle.download_c2c_record_by_uuid(Some(uuid.clone())).await
                        }
                        _ => continue,
                    },
                    _ => {
                        tracing::error!("{:?} Missing msg_info or audio_uuid!", record);
                        continue;
                    }
                };
                record.audio_url = match url_result {
                    Ok(url) => url,
                    Err(e) => {
                        tracing::error!("Failed to download record: {:?}", e);
                        continue;
                    }
                }
            }
            Entity::Video(ref mut video) => {
                let file_name = video.file_name.as_ref();
                let empty_option = Some(String::new());
                let node = video.node.clone();
                let download_result = match &chain.typ {
                    MessageType::Group(grp) => {
                        let uid = handle
                            .uin2uid_fast(chain.friend_uin, Some(grp.group_uin))
                            .await;
                        match handle
                            .download_video(
                                &uid,
                                file_name,
                                "",
                                empty_option.clone(),
                                node.clone(),
                                true,
                            )
                            .await
                        {
                            Ok(url) => Ok(url),
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to download group video: {:?}, using download_group_video fallback!",
                                    e
                                );
                                handle
                                    .download_group_video(
                                        grp.group_uin,
                                        file_name,
                                        "",
                                        empty_option,
                                        node.clone(),
                                    )
                                    .await
                            }
                        }
                    }
                    MessageType::Friend(_) | MessageType::Temp => {
                        let self_uid = chain.uid.clone();
                        handle
                            .download_video(
                                &self_uid,
                                file_name,
                                "",
                                empty_option,
                                node.clone(),
                                false,
                            )
                            .await
                    }
                    _ => continue,
                };
                match download_result {
                    Ok(url) => {
                        video.video_url = url;
                    }
                    Err(e) => {
                        tracing::error!("Failed to download video: {:?}", e);
                    }
                }
            }
            _ => {}
        }
    }
}

async fn messaging_logic_outgoing(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(send) = event.as_any_mut().downcast_mut::<SendMessageEvent>() => {
            resolve_chain_metadata(&mut send.chain, handle.clone()).await;
            resolve_outgoing_chain(&mut send.chain, handle.clone()).await;
            // TODO: await Collection.Highway.UploadResources(send.Chain);
        }
        _ => {}
    }
    event
}

async fn resolve_outgoing_chain(_: &mut MessageChain, _: Arc<BusinessHandle>) {}

// TODO: return result!!!
async fn resolve_chain_metadata(
    chain: &mut MessageChain,
    handle: Arc<BusinessHandle>,
) -> &mut MessageChain {
    match chain.typ {
        MessageType::Group(ref mut grp)
            if handle.context.config.fetch_group_member_strategy
                == FetchGroupMemberStrategy::Full =>
        {
            let members = handle
                .fetch_maybe_cached_group_members(
                    grp.group_uin,
                    |mm| {
                        mm.get(&grp.group_uin)
                            .map(|entry| entry.value().clone())
                            .unwrap_or_else(|| {
                                tracing::warn!(
                                    "No group members found for group: {}",
                                    grp.group_uin
                                );
                                Vec::new()
                            })
                    },
                    false,
                )
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to fetch group members: {:?}", e);
                    Vec::new()
                });
            let lookup_uin = if chain.friend_uin == 0 {
                **handle.context.key_store.uin.load()
            } else {
                chain.friend_uin
            };
            grp.group_member_info = members.into_iter().find(|member| member.uin == lookup_uin);
            if chain.uid.is_empty()
                && let Some(member) = &grp.group_member_info
            {
                chain.uid = member.uid.clone();
            }
            chain
        }

        // TODO: optimization
        MessageType::Friend(ref mut friend_elem) => {
            let friends = handle
                .fetch_maybe_cached_friends(
                    Some(chain.friend_uin),
                    |fm| fm.iter().map(|entry| entry.value().clone()).collect(),
                    false,
                )
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to fetch friends: {:?}", e);
                    Vec::new()
                });
            if let Some(friend) = friends
                .into_iter()
                .find(|friend| friend.uin == chain.friend_uin)
            {
                friend_elem.friend_info = Some(friend.clone());
                if chain.uid.is_empty() {
                    chain.uid = friend.uid.clone();
                }
            }
            chain
        }
        _ => chain,
    }
}
