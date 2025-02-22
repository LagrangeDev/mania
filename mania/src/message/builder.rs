use crate::core::highway::{AsyncPureStream, AsyncPureStreamTrait};
use crate::dda;
use crate::message::chain::MessageChain;
use crate::message::entity::Entity;
use crate::message::entity::image::ImageEntity;
use crate::message::entity::record::RecordEntity;
use crate::message::entity::text::TextEntity;
use crate::message::entity::video::VideoEntity;
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

    pub fn video(&mut self, video_path: &str, video_length: i32) -> &mut Self {
        self.chains.entities.push(Entity::Video(dda!(VideoEntity {
            video_path: Some(video_path.to_string()),
            video_length,
        })));
        self
    }

    pub fn video_with_thumb(
        &mut self,
        video_path: &str,
        video_length: i32,
        thumb_path: &str,
    ) -> &mut Self {
        self.chains.entities.push(Entity::Video(dda!(VideoEntity {
            video_path: Some(video_path.to_string()),
            video_length,
            video_thumb_path: Some(thumb_path.to_string()),
        })));
        self
    }

    pub fn video_stream(
        &mut self,
        video_stream: impl AsyncPureStreamTrait + 'static,
        video_length: i32,
    ) -> &mut Self {
        self.chains.entities.push(Entity::Video(dda!(VideoEntity {
            video_stream: Some(Arc::new(Mutex::new(
                Box::new(video_stream) as AsyncPureStream
            ))),
            video_length
        })));
        self
    }

    pub fn video_stream_with_thumb(
        &mut self,
        video_stream: impl AsyncPureStreamTrait + 'static,
        video_length: i32,
        thumb_stream: impl AsyncPureStreamTrait + 'static,
    ) -> &mut Self {
        self.chains.entities.push(Entity::Video(dda!(VideoEntity {
            video_stream: Some(Arc::new(Mutex::new(
                Box::new(video_stream) as AsyncPureStream
            ))),
            video_thumb_stream: Some(Arc::new(Mutex::new(
                Box::new(thumb_stream) as AsyncPureStream
            ))),
            video_length
        })));
        self
    }

    pub fn record(&mut self, record_path: &str) -> &mut Self {
        self.chains.entities.push(Entity::Record(dda!(RecordEntity {
            file_path: Some(record_path.to_string()),
        })));
        self
    }

    pub fn record_stream(
        &mut self,
        record_stream: impl AsyncPureStreamTrait + 'static,
    ) -> &mut Self {
        self.chains.entities.push(Entity::Record(dda!(RecordEntity {
            audio_stream: Some(Arc::new(Mutex::new(
                Box::new(record_stream) as AsyncPureStream
            ))),
        })));
        self
    }

    pub fn build(&mut self) -> MessageChain {
        std::mem::take(&mut self.chains)
    }
}
