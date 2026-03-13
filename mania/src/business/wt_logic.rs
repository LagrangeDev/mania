use crate::business::{BusinessHandle, BusinessResult, LogicFlow};
use crate::event::system::SystemEvent;
use crate::event::system::bot_offline::BotOfflineEvent;
use mania_core::core::event::prelude::*;
use mania_core::core::event::system::kick_nt::KickNTEvent;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(KickNTEvent)]
async fn messaging_logic(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
    flow: LogicFlow,
) -> BusinessResult<&dyn ServerEvent> {
    match flow {
        LogicFlow::InComing => Ok(messaging_logic_incoming(event, handle).await),
        LogicFlow::OutGoing => Ok(event),
    }
}

async fn messaging_logic_incoming(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(kick) = event.as_any_mut().downcast_mut::<KickNTEvent>() => {
            tracing::error!("KickNTEvent: {:?}", kick);
            tracing::error!("Bot will be offline in 5 seconds...");
            handle
                .event_dispatcher
                .system
                .send(Some(SystemEvent::BotOfflineEvent(BotOfflineEvent {
                    reason: Some(format!("{}: {}", kick.title, kick.tips)),
                })))
                .expect("send BotOnlineEvent failed");
        }
        _ => {}
    }
    event
}
