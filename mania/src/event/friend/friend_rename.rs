pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct FriendRenameEvent {
    pub uin: u32,
    pub nickname: String,
}
