use crate::core::business::LogicRegistry;
use crate::core::business::{BusinessHandle, LogicFlow};
use crate::core::event::message::push_msg::PushMessageEvent;
use crate::core::event::notify::group_sys_request_join::GroupSysRequestJoinEvent;
use crate::core::event::prelude::*;
use crate::event::group::{GroupEvent, group_message};
use crate::message::chain::{MessageChain, MessageType};
use crate::message::entity::Entity;
use crate::message::entity::file::FileUnique;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(PushMessageEvent, GroupSysRequestJoinEvent)]
async fn messaging_logic(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
    flow: LogicFlow,
) -> &dyn ServerEvent {
    tracing::debug!("[{}] Handling event: {:?}", flow, event);
    match flow {
        LogicFlow::InComing => messaging_logic_incoming(event, handle).await,
        LogicFlow::OutGoing => messaging_logic_outgoing(event, handle).await,
    }
}

async fn messaging_logic_incoming(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(msg) = event.as_any_mut().downcast_mut::<PushMessageEvent>() => {
            if let Some(mut chain) = msg.chain.take() {
                resolve_incoming_chain(&mut chain, handle.clone()).await;
                // TODO: await ResolveChainMetadata(push.Chain);
                // TODO: MessageFilter.Filter(push.Chain);
                // TODO?: sb tx! Collection.Invoker.PostEvent(new GroupInvitationEvent(groupUin, chain.FriendUin, sequence));
                match &chain.typ {
                    MessageType::Group(_) => {
                        handle
                            .event_dispatcher
                            .group
                            .send(Some(GroupEvent::GroupMessage(
                                group_message::GroupMessageEvent { chain },
                            )))
                            .expect("Failed to send group event");
                    }
                    // TODO: friend message & temp message
                    _ => {}
                }
            } else {
                tracing::warn!("Empty message chain in PushMessageEvent");
            }
        }
        _ if let Some(event) = event
            .as_any_mut()
            .downcast_mut::<GroupSysRequestJoinEvent>() =>
        {
            tracing::debug!("Handling GroupSysRequestJoinEvent: {:?}", event); // TODO: dispatch
        }
        _ => {}
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
                    None => continue,
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
                            .resolve_uid(Some(grp.group_uin), chain.friend_uin)
                            .await;
                        let uid = uid.unwrap_or_default();
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
