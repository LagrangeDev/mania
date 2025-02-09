pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupMuteEvent {
    pub group_uin: u32,
    pub operator_uin: Option<u32>,
    pub is_muted: bool,
}
