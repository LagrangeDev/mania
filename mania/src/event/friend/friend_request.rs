pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct FriendRequestEvent {
    pub source_uin: u32,
    pub message: String,
    pub source: String,
}
