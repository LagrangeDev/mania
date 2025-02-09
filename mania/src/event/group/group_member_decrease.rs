pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupMemberDecreaseEvent {
    pub group_uin: u32,
    pub member_uin: u32,
    pub operator_uin: Option<u32>,
    pub event_type: EventType,
}

#[derive(Debug)]
pub enum EventType {
    KickMe = 3,
    Disband = 129,
    Leave = 130,
    Kick = 131,
}
