pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupMemberIncreaseEvent {
    pub group_uin: u32,
    pub member_uin: u32,
    pub invitor_uin: Option<u32>,
    pub event_type: EventType,
}

#[derive(Debug)]
pub enum EventType {
    Approve = 130,
    Invite = 131,
}
