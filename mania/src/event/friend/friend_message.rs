pub use mania_macros::ManiaEvent;

use crate::message::chain::MessageChain;

#[derive(ManiaEvent)]
pub struct FriendMessageEvent {
    pub chain: MessageChain,
}
