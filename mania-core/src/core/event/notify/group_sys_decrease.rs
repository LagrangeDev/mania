use crate::core::entity::group_sys_enum::GroupMemberDecreaseEventType;
use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysDecreaseEvent {
    pub group_uin: u32,
    pub member_uid: String,
    pub operator_uid: Option<String>,
    pub event_type: GroupMemberDecreaseEventType,
}
