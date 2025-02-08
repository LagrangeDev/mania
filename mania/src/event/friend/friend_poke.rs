pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct FriendPokeEvent {
    pub operator_uin: u32,
    pub target_uin: u32,
    pub action: String,
    pub suffix: String,
    pub action_url: String,
}
