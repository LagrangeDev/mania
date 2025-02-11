use crate::core::entity::group_sys_enum::GroupMemberIncreaseEventType;
use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysIncreaseEvent {
    pub group_uin: u32,
    pub member_uid: String,
    pub invitor_uid: Option<String>,
    pub event_type: GroupMemberIncreaseEventType,
}
