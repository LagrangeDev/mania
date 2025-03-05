use std::sync::Arc;

use mania_macros::handle_event;

use crate::{
    core::{
        business::{BusinessError, BusinessHandle, LogicFlow, LogicRegistry},
        event::{prelude::*, system::kick_nt::KickNTEvent},
    },
    event::system::{SystemEvent, bot_offline::BotOfflineEvent},
};

#[handle_event(KickNTEvent)]
async fn messaging_logic(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
    flow: LogicFlow,
) -> Result<&dyn ServerEvent, BusinessError> {
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
