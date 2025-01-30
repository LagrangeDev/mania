use crate::core::business::LogicRegistry;
use crate::core::business::{BusinessHandle, LogicFlow};
use crate::core::event::prelude::*;
use crate::core::event::system::alive::AliveEvent;
use crate::core::event::system::info_sync::InfoSyncEvent;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(AliveEvent, InfoSyncEvent)]
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
    _: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(alive) = event.as_any_mut().downcast_mut::<AliveEvent>() => {
            alive.test = 114514; // FIXME: This is a test
            // handle.fetch_rkey().await.unwrap();
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
