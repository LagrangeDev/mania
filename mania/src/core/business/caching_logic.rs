use crate::core::business::{BusinessError, LogicRegistry};
use crate::core::business::{BusinessHandle, LogicFlow};
use crate::core::event::notify::group_sys_decrease::GroupSysDecreaseEvent;
use crate::core::event::notify::group_sys_increase::GroupSysIncreaseEvent;
use crate::core::event::prelude::*;
use mania_macros::handle_event;
use std::sync::Arc;

#[handle_event(GroupSysIncreaseEvent, GroupSysDecreaseEvent)]
async fn caching_logic(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
    flow: LogicFlow,
) -> Result<&dyn ServerEvent, BusinessError> {
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
