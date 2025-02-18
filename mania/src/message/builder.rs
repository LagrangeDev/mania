use crate::core::highway::{AsyncPureStream, AsyncPureStreamTrait};
use crate::dda;
use crate::message::chain::MessageChain;
use crate::message::entity::Entity;
use crate::message::entity::image::ImageEntity;
use crate::message::entity::text::TextEntity;
use std::sync::Arc;
use tokio::sync::Mutex;

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
        self.chains.entities.push(Entity::Text(TextEntity {
            text: content.to_string(),
        }));
        self
    }

    pub fn image(&mut self, image_path: &str) -> &mut Self {
        self.chains.entities.push(Entity::Image(dda!(ImageEntity {
            file_path: Some(image_path.to_string()),
        })));
        self
    }

    pub fn image_stream(
        &mut self,
        image_stream: impl AsyncPureStreamTrait + 'static,
        size: u32,
    ) -> &mut Self {
        self.chains.entities.push(Entity::Image(dda!(ImageEntity {
            image_stream: Some(Arc::new(Mutex::new(
                Box::new(image_stream) as AsyncPureStream
            ))),
            size,
        })));
        self
    }

    pub fn build(&mut self) -> MessageChain {
        std::mem::take(&mut self.chains)
    }
}
