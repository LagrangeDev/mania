pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupInvitationEvent {
    pub group_uin: u32,
    pub invitor_uin: u32,
    pub sequence: Option<u64>,
}
