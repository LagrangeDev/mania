pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupJoinRequestEvent {
    pub group_uin: u32,
    pub target_uin: u32,
    pub target_nickname: String,
    pub invitor_uin: u32,
    pub answer: String,
    pub request_seq: u64,
}
