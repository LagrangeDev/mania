pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupMemberMuteEvent {
    pub group_uin: u32,
    pub target_uin: u32,
    pub operator_uin: Option<u32>,
    pub duration: u32,
}
