pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupJoinRequestEvent {
    pub target_uin: u32,
    pub group_uin: u32,
}
