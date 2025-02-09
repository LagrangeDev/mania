pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupInvitationRequestEvent {
    pub group_uin: u32,
    pub target_uin: u32,
    pub invitor_uin: u32,
}
