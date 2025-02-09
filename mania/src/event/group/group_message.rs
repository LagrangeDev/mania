use crate::message::chain::MessageChain;
pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupMessageEvent {
    pub chain: MessageChain,
}
