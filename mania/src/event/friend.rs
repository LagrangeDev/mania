pub mod friend_message;
pub mod friend_new;
pub mod friend_poke;
pub mod friend_recall;
pub mod friend_rename;
pub mod friend_request;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // FIXME: do we need spilt or refactoring?
pub enum FriendEvent {
    FriendPokeEvent(friend_poke::FriendPokeEvent), // FIXME: clippy warn: at least 80 bytes
    FriendMessageEvent(friend_message::FriendMessageEvent), // FIXME: clippy warn: at least 320 bytes
    FriendRecallEvent(friend_recall::FriendRecallEvent),
    FriendRequestEvent(friend_request::FriendRequestEvent),
    FriendRenameEvent(friend_rename::FriendRenameEvent),
    FriendNewEvent(friend_new::FriendNewEvent),
}
