use crate::core::business::LogicRegistry;
use crate::core::business::{BusinessHandle, LogicFlow};
use crate::core::event::message::push_msg::PushMessageEvent;
use crate::core::event::prelude::*;
use crate::core::event::system::alive::AliveEvent;
use crate::core::event::system::info_sync::InfoSyncEvent;
use crate::message::chain::MessageType;
use crate::message::entity::file::FileUnique;
use crate::message::entity::Entity;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(AliveEvent, InfoSyncEvent, PushMessageEvent)]
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

#[allow(clippy::single_match)] // TODO: remove when finally implemented
async fn messaging_logic_incoming(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(msg) = event.as_any_mut().downcast_mut::<PushMessageEvent>() => {
            match &mut msg.chain {
                Some(chain) => {
                    for entity in &mut chain.entities {
                        match entity {
                            Entity::Image(ref mut image) => {
                                if !image.url.contains("&rkey=") {
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
                            Entity::MultiMsg(ref mut multi) => match multi.chains.is_empty() {
                                true => {
                                    let msg = handle
                                        .multi_msg_download(chain.uid.clone(), multi.res_id.clone())
                                        .await;
                                    match msg {
                                        Ok(Some(chains)) => {
                                            multi.chains = chains;
                                        }
                                        Ok(None) => {
                                            tracing::warn!(
                                                "No chains found in MultiMsgDownloadEvent"
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to download MultiMsg: {:?}", e);
                                        }
                                    }
                                }
                                false => {}
                            },
                            Entity::File(ref mut file) => {
                                file.file_url = match file.extra.as_ref() {
                                    Some(extra) => match extra {
                                        FileUnique::Group(grp) => {
                                            let group_uin =
                                                if let MessageType::Group(ref grp) = chain.typ {
                                                    grp.group_uin
                                                } else {
                                                    tracing::error!(
                                                        "expected group message, find {:?}",
                                                        chain.typ
                                                    );
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
                                                .download_group_record_by_node(
                                                    grp.group_uin,
                                                    node(),
                                                )
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
                                                .download_group_record_by_uuid(
                                                    grp.group_uin,
                                                    Some(uuid.clone()),
                                                )
                                                .await
                                        }
                                        MessageType::Friend(_) | MessageType::Temp => {
                                            handle
                                                .download_c2c_record_by_uuid(Some(uuid.clone()))
                                                .await
                                        }
                                        _ => continue,
                                    },
                                    _ => {
                                        tracing::error!(
                                            "{:?} Missing msg_info or audio_uuid!",
                                            record
                                        );
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
                            _ => {}
                        }
                    }
                }
                None => return event,
            }
        }
        _ if let Some(info_sync) = event.as_any_mut().downcast_mut::<InfoSyncEvent>() => {
            tracing::debug!("Handling InfoSync event: {:?}", info_sync);
        }
        _ => {}
    }
    event
}

async fn messaging_logic_outgoing(
    event: &mut dyn ServerEvent,
    _: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(alive) = event.as_any_mut().downcast_mut::<AliveEvent>() => {
            alive.test = 114514; // FIXME: This is a test
        }
        _ if let Some(info_sync) = event.as_any_mut().downcast_mut::<InfoSyncEvent>() => {
            tracing::debug!("Handling InfoSync event: {:?}", info_sync);
        }
        _ => {}
    }
    event
}
