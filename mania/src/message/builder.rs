use crate::message::chain::MessageChain;
use crate::message::entity::{Entity, text};

#[derive(Default)]
pub struct MessageChainBuilder {
    chains: MessageChain,
}

impl MessageChainBuilder {
    pub fn friend(friend_uin: u32) -> Self {
        Self {
            chains: MessageChain::friend(friend_uin, "", ""),
        }
    }

    pub fn group(group_uin: u32) -> Self {
        Self {
            chains: MessageChain::group(group_uin),
        }
    }

    pub fn text(&mut self, content: &str) -> &mut Self {
        self.chains.entities.push(Entity::Text(text::TextEntity {
            text: content.to_string(),
        }));
        self
    }

    pub fn build(&mut self) -> MessageChain {
        std::mem::take(&mut self.chains)
    }
}
