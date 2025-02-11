use crate::core::entity::group_sys_enum::GroupMemberIncreaseEventType;
pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupMemberIncreaseEvent {
    pub group_uin: u32,
    pub member_uin: u32,
    pub invitor_uin: Option<u32>,
    pub event_type: GroupMemberIncreaseEventType,
}
