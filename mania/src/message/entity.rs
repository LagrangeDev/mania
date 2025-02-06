pub mod face;
pub mod file;
pub mod forward;
pub mod image;
pub mod json;
pub mod light_app;
pub mod market_face;
pub mod mention;
pub mod multi_msg;
pub mod record;
pub mod text;

pub use face::FaceEntity as Face;
pub use file::FileEntity as File;
pub use forward::ForwardEntity as Forward;
pub use image::ImageEntity as Image;
pub use json::JsonEntity as Json;
pub use light_app::LightAppEntity as LightApp;
pub use market_face::MarketFaceEntity as MarketFace;
pub use mention::MentionEntity as Mention;
pub use multi_msg::MultiMsgEntity as MultiMsg;
pub use record::RecordEntity as Record;
pub use text::TextEntity as Text;

use crate::core::protos::message::Elem;
use bytes::Bytes;
use std::fmt::{Debug, Display};

pub trait MessageEntity: Debug + Display {
    fn pack_element(&self) -> Vec<Elem>;
    fn pack_content(&self) -> Option<Bytes> {
        None
    }
    fn unpack_element(elem: &Elem) -> Option<Self>
    where
        Self: Sized;
}

pub enum Entity {
    Text(text::TextEntity),
    Json(json::JsonEntity),
    Image(image::ImageEntity),
    Face(face::FaceEntity),
    Forward(forward::ForwardEntity),
    MarketFace(market_face::MarketFaceEntity),
    LightApp(light_app::LightAppEntity),
    MultiMsg(multi_msg::MultiMsgEntity),
    Mention(mention::MentionEntity),
    File(file::FileEntity),
    Record(record::RecordEntity),
}

macro_rules! impl_entity_show {
    ( $( $variant:ident ),* $(,)? ) => {
        impl std::fmt::Debug for Entity {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Entity::$variant(inner) => write!(f, "{:?}", inner),
                    )*
                }
            }
        }
        impl std::fmt::Display for Entity {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Entity::$variant(inner) => write!(f, "{}", inner),
                    )*
                }
            }
        }
    }
}

macro_rules! impl_entity_pack {
    ( $( $variant:ident ),* $(,)? ) => {
        impl Entity {
            pub fn pack_element(&self) -> Vec<Elem> {
                match self {
                    $(
                        Entity::$variant(inner) => inner.pack_element(),
                    )*
                }
            }
            pub fn pack_content(&self) -> Option<Bytes> {
                match self {
                    $(
                        Entity::$variant(inner) => inner.pack_content(),
                    )*
                }
            }
        }
    }
}

macro_rules! impl_entity_unpack {
    ( $( $variant:ident ),* $(,)? ) => {
        impl Entity {
            pub fn unpack_element(elem: &Elem) -> Option<Self> {
                $(
                    if let Some(inner) = <$crate::message::entity::$variant as MessageEntity>::unpack_element(elem) {
                        return Some(Entity::$variant(inner));
                    }
                )*
                None
            }
        }
    }
}

impl_entity_show!(
    Text, Json, Image, Face, Forward, MarketFace, LightApp, MultiMsg, Mention, File, Record
);
impl_entity_pack!(
    Text, Json, Image, Face, Forward, MarketFace, LightApp, MultiMsg, Mention, File, Record
);
impl_entity_unpack!(
    Text, Json, Image, Face, Forward, MarketFace, LightApp, MultiMsg, Mention, File, Record
);

impl Entity {
    pub fn from_elems(elems: &[Elem]) -> Vec<Self> {
        elems.iter().filter_map(Entity::unpack_element).collect()
    }
    pub fn to_elems(&self) -> Vec<Elem> {
        self.pack_element().into_iter().collect()
    }
}

mod prelude {
    pub use crate::core::protos::message::*;
    pub use crate::dda;
    pub use crate::message::chain::{ClientSequence, MessageId};
    pub use crate::message::entity::MessageEntity;
    pub use crate::utility::compress::*;
    pub use bytes::Bytes;
    pub use chrono::{DateTime, Utc};
    pub use prost::Message;
    pub use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
    pub use std::io::{Read, Write};
}
