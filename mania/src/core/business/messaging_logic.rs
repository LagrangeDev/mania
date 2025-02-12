use crate::core::business::LogicRegistry;
use crate::core::business::{BusinessHandle, LogicFlow};
use crate::core::event::message::push_msg::PushMessageEvent;
use crate::core::event::notify::friend_sys_poke::FriendSysPokeEvent;
use crate::core::event::notify::friend_sys_recall::FriendSysRecallEvent;
use crate::core::event::notify::group_sys_decrease::GroupSysDecreaseEvent;
use crate::core::event::notify::group_sys_increase::GroupSysIncreaseEvent;
use crate::core::event::notify::group_sys_poke::GroupSysPokeEvent;
use crate::core::event::notify::group_sys_reaction::GroupSysReactionEvent;
use crate::core::event::notify::group_sys_recall::GroupSysRecallEvent;
use crate::core::event::notify::group_sys_request_join::GroupSysRequestJoinEvent;
use crate::core::event::prelude::*;
use crate::event::friend::friend_poke::FriendPokeEvent;
use crate::event::friend::{FriendEvent, friend_message, friend_recall};
use crate::event::group::group_poke::GroupPokeEvent;
use crate::event::group::group_reaction::GroupReactionEvent;
use crate::event::group::group_recall::GroupRecallEvent;
use crate::event::group::{
    GroupEvent, group_join_request, group_member_decrease, group_member_increase, group_message,
};
use crate::event::system::{SystemEvent, temp_message};
use crate::message::chain::{MessageChain, MessageType};
use crate::message::entity::Entity;
use crate::message::entity::file::FileUnique;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(
    PushMessageEvent,
    GroupSysRequestJoinEvent,
    GroupSysPokeEvent,
    GroupSysReactionEvent,
    GroupSysRecallEvent,
    GroupSysIncreaseEvent,
    GroupSysDecreaseEvent,
    FriendSysRecallEvent,
    FriendSysPokeEvent
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
                // TODO: await ResolveChainMetadata(push.Chain);
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
                tracing::warn!("Empty message chain in PushMessageEvent");
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
    }
    {
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
            let member_uin = handle
                .resolve_stranger_uid2uin(&join.member_uid)
                .await
                .unwrap_or_default();
            let invitor_uin = handle
                .uid2uin(
                    join.invitor_uid.as_deref().unwrap_or(""),
                    Some(join.group_uin),
                )
                .await
                .ok();
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
                .resolve_stranger_uid2uin(&leave.member_uid)
                .await
                .unwrap_or_default();
            let operator_uin = handle
                .uid2uin(
                    leave.operator_uid.as_deref().unwrap_or(""),
                    Some(leave.group_uin),
                )
                .await
                .ok();
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
        if let Some(reaction) = event.as_any_mut().downcast_mut::<GroupSysReactionEvent>() {
            if let Err(e) = handle
                .event_dispatcher
                .group
                .send(Some(GroupEvent::GroupReaction(GroupReactionEvent {
                    target_group_uin: reaction.target_group_uin,
                    target_sequence: reaction.target_sequence,
                    operator_uin: handle
                        .uid2uin(&reaction.operator_uid, Some(reaction.target_group_uin))
                        .await
                        .unwrap_or_default(),
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
                        .uid2uin(&recall.author_uid, Some(recall.group_uin))
                        .await
                        .unwrap_or_default(),
                    operator_uin: if let Some(uid) = recall.operator_uid.as_ref() {
                        handle
                            .uid2uin(uid, Some(recall.group_uin))
                            .await
                            .unwrap_or_default()
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
        if let Some(recall) = event.as_any_mut().downcast_mut::<FriendSysRecallEvent>() {
            if let Err(e) =
                handle
                    .event_dispatcher
                    .friend
                    .send(Some(FriendEvent::FriendRecallEvent(
                        friend_recall::FriendRecallEvent {
                            friend_uin: handle
                                .uid2uin(&recall.from_uid, None)
                                .await
                                .unwrap_or_default(),
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
                            .uin2uid(chain.friend_uin, Some(grp.group_uin))
                            .await
                            .unwrap_or_default();
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
    _: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    event
}
