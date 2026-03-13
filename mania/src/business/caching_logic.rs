use crate::business::{BusinessHandle, BusinessResult, LogicFlow};
use mania_core::core::event::notify::group_sys_decrease::GroupSysDecreaseEvent;
use mania_core::core::event::notify::group_sys_increase::GroupSysIncreaseEvent;
use mania_core::core::event::prelude::*;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(GroupSysIncreaseEvent, GroupSysDecreaseEvent)]
async fn caching_logic(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
    flow: LogicFlow,
) -> BusinessResult<&dyn ServerEvent> {
    match flow {
        LogicFlow::InComing => Ok(caching_logic_incoming(event, handle).await),
        LogicFlow::OutGoing => Ok(event),
    }
}

async fn caching_logic_incoming(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
) -> &dyn ServerEvent {
    match event {
        _ if let Some(increase) = event.as_any_mut().downcast_mut::<GroupSysIncreaseEvent>() => {
            tracing::info!(
                "caching_logic_incoming GroupSysIncreaseEvent: {:?}",
                increase
            );
            if let Err(e) = handle.refresh_group_members_cache(increase.group_uin).await {
                tracing::error!("refresh_group_members_cache failed: {:?}", e);
            }
        }
        _ if let Some(decrease) = event.as_any_mut().downcast_mut::<GroupSysDecreaseEvent>() => {
            tracing::info!(
                "caching_logic_incoming GroupSysDecreaseEvent: {:?}",
                decrease
            );
            let self_uid = handle
                .context
                .session
                .key_store
                .uid
                .load()
                .as_ref()
                .expect("Missing self_uid")
                .as_ref()
                .clone();
            if decrease.member_uid != self_uid
                && let Err(e) = handle.refresh_group_members_cache(decrease.group_uin).await
            {
                tracing::error!("refresh_group_members_cache failed: {:?}", e);
            }
        }
        _ => {}
    }
    event
}
