pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct FriendRecallEvent {
    pub friend_uin: u32,
    pub client_sequence: u32,
    pub time: u32,
    pub random: u32,
    pub tip: String,
}
