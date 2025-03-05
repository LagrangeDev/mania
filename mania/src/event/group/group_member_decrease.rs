pub use mania_macros::ManiaEvent;

use crate::core::entity::group_sys_enum::GroupMemberDecreaseEventType;

#[derive(ManiaEvent)]
pub struct GroupMemberDecreaseEvent {
    pub group_uin: u32,
    pub member_uin: u32,
    pub operator_uin: Option<u32>,
    pub event_type: GroupMemberDecreaseEventType,
}
