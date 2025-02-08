use crate::event::prelude::*;

#[derive(ManiaEvent)]
pub struct FriendPokeEvent {
    pub operator_uin: u32,
    pub target_uin: u32,
    pub action: String,
    pub suffix: String,
    pub action_url: String,
}

impl Debug for FriendPokeEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[FriendPokeEvent]: operator_uin: {} | target_uin: {} | action: {} | suffix: {} | action_url: {}",
            self.operator_uin, self.target_uin, self.action, self.suffix, self.action_url
        )
    }
}
