pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct FriendNewEvent {
    pub from_uin: u32,
    pub from_nickname: String,
    pub msg: String,
}
