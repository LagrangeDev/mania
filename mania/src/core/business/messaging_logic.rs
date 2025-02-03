use crate::core::business::LogicRegistry;
use crate::core::business::{BusinessHandle, LogicFlow};
use crate::core::event::message::push_msg::PushMessageEvent;
use crate::core::event::prelude::*;
use crate::core::event::system::alive::AliveEvent;
use crate::core::event::system::info_sync::InfoSyncEvent;
use crate::message::chain::MessageType;
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
            let chain = &mut msg.chain;
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
                    _ => {}
                }
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
