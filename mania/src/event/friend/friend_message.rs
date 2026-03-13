use mania_core::message::chain::MessageChain;
use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct FriendMessageEvent {
    pub chain: MessageChain,
}
