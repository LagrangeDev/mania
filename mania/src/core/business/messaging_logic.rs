use crate::core::business::LogicFlow;
use crate::core::business::LogicRegistry;
use crate::core::event::prelude::*;
use crate::core::event::system::alive::Alive;
use crate::core::event::system::info_sync::InfoSync;
use crate::Context;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(Alive, InfoSync)]
async fn messaging_logic(
    event: &mut dyn ServerEvent,
    ctx: Arc<Context>,
    flow: LogicFlow,
) -> &dyn ServerEvent {
    tracing::debug!("[{}] Handling event: {:?}", flow, event);
    match flow {
        LogicFlow::InComing => messaging_logic_incoming(event, ctx).await,
        LogicFlow::OutGoing => messaging_logic_outgoing(event, ctx).await,
    }
}

async fn messaging_logic_incoming(
    event: &mut dyn ServerEvent,
    _: Arc<Context>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(alive) = event.as_any_mut().downcast_mut::<Alive>() => {
            alive.test = 114514;
            tracing::debug!("Handling Alive event: {:?}", alive);
        }
        _ if let Some(info_sync) = event.as_any_mut().downcast_mut::<InfoSync>() => {
            tracing::debug!("Handling InfoSync event: {:?}", info_sync);
        }
        _ => {}
    }
    event
}

async fn messaging_logic_outgoing(
    event: &mut dyn ServerEvent,
    _: Arc<Context>,
) -> &dyn ServerEvent {
    event
}
