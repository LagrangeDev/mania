pub mod friend_message;
pub mod friend_poke;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // FIXME: do we need spilt or refactoring?
pub enum FriendEvent {
    FriendPokeEvent(friend_poke::FriendPokeEvent), // FIXME: clippy warn: at least 80 bytes
    FriendMessageEvent(friend_message::FriendMessageEvent), // FIXME: clippy warn: at least 320 bytes
}
