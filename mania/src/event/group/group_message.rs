use crate::event::prelude::*;
use crate::message::chain::MessageChain;

#[derive(ManiaEvent)]
pub struct GroupMessageEvent {
    pub chain: MessageChain,
}

impl Debug for GroupMessageEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[GroupMessageEvent]: {:?}", self.chain)
    }
}
