pub mod friend_poke;

#[derive(Debug)]
pub enum FriendEvent {
    FriendPokeEvent(friend_poke::FriendPokeEvent),
}
