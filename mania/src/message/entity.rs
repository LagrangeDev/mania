pub mod extra_general_flags;
pub mod extra_info;
pub mod face;
pub mod file;
pub mod forward;
pub mod image;
pub mod json;
pub mod light_app;
pub mod long_msg;
pub mod market_face;
pub mod mention;
pub mod multi_msg;
pub mod record;
pub mod text;
pub mod video;
pub mod xml;

pub use extra_general_flags::ExtraGeneralFlagsEntity as ExtraGeneralFlags;
pub use extra_info::ExtraInfoEntity as ExtraInfo;
pub use face::FaceEntity as Face;
pub use file::FileEntity as File;
pub use forward::ForwardEntity as Forward;
pub use image::ImageEntity as Image;
pub use json::JsonEntity as Json;
pub use light_app::LightAppEntity as LightApp;
pub use long_msg::LongMsgEntity as LongMsg;
pub use market_face::MarketFaceEntity as MarketFace;
pub use mention::MentionEntity as Mention;
pub use multi_msg::MultiMsgEntity as MultiMsg;
pub use record::RecordEntity as Record;
pub use text::TextEntity as Text;
pub use video::VideoEntity as Video;
pub use xml::XmlEntity as Xml;

use crate::core::protos::message::Elem;
use bytes::Bytes;
use std::fmt::{Debug, Display};

pub trait MessageContentImplChecker {
    fn need_pack(&self) -> bool;
}

pub trait MessageContentImpl: MessageContentImplChecker {
    fn pack_content(&self) -> Option<Bytes>;
}

pub trait MessageEntity: Debug + Display + MessageContentImpl {
    fn pack_element(&self, self_uid: &str) -> Vec<Elem>;
    fn unpack_element(elem: &Elem) -> Option<Self>
    where
        Self: Sized;
}

#[allow(clippy::large_enum_variant)] // FIXME: do we need refactoring?
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
    Video(video::VideoEntity), // FIXME: clippy warn: at least 800 bytes
    Xml(xml::XmlEntity),
    LongMsg(long_msg::LongMsgEntity), // FIXME: clippy warn: at least 344 bytes
    ExtraInfo(extra_info::ExtraInfoEntity),
    ExtraGeneralFlags(extra_general_flags::ExtraGeneralFlagsEntity),
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
            pub fn need_pack(&self) -> bool {
                match self {
                    $(
                        Entity::$variant(inner) => inner.need_pack(),
                    )*
                }
            }
            pub fn pack_element(&self, self_uid: &str) -> Vec<Elem> {
                match self {
                    $(
                        Entity::$variant(inner) => inner.pack_element(self_uid),
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

macro_rules! impl_common_entity_unpack {
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

macro_rules! impl_extra_entity_unpack {
    ( $( $variant:ident ),* $(,)? ) => {
        impl Entity {
            pub fn unpack_extra_element(elem: &Elem) -> Option<Self> {
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

macro_rules! impl_show_pack_all {
    ( $( $variant:ident ),* $(,)? ) => {
        impl_entity_show!( $( $variant ),* );
        impl_entity_pack!( $( $variant ),* );
    };
}

impl_show_pack_all!(
    Text,
    Json,
    Image,
    Face,
    Forward,
    MarketFace,
    LightApp,
    MultiMsg,
    Mention,
    File,
    Record,
    Video,
    Xml,
    LongMsg,
    ExtraInfo,
    ExtraGeneralFlags
);

impl_common_entity_unpack!(
    Text, Json, Image, Face, Forward, MarketFace, LightApp, MultiMsg, Mention, File, Record, Video,
    Xml, LongMsg
);

impl_extra_entity_unpack!(ExtraInfo, ExtraGeneralFlags);

impl Entity {
    pub fn from_elems(elems: &[Elem]) -> Vec<Self> {
        elems.iter().filter_map(Entity::unpack_element).collect()
    }

    pub fn to_elems(&self, self_uid: &str) -> Vec<Elem> {
        self.pack_element(self_uid).into_iter().collect()
    }

    pub fn need_pack_content(elems: &[Self]) -> bool {
        elems.iter().any(|e| e.need_pack())
    }
}

mod prelude {
    pub use crate::core::protos::message::*;
    pub use crate::dda;
    pub use crate::message::chain::{ClientSequence, MessageId};
    pub use crate::message::entity::{MessageContentImpl, MessageEntity};
    pub use crate::utility::compress::*;
    pub use bytes::Bytes;
    pub use chrono::{DateTime, Utc};
    pub use mania_macros::pack_content;
    pub use prost::Message;
    pub use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
    pub use std::io::{Read, Write};
}
