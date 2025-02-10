pub mod action;
pub mod login;
pub mod message;
pub mod notify;
pub mod system;

use crate::core::context::Context;
use crate::core::packet::{BinaryPacket, PacketReader, PacketType, SsoPacket};
use bytes::Bytes;
use once_cell::sync::Lazy;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

pub trait ServerEvent: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait CECommandMarker: Send + Sync {
    const COMMAND: &'static str;
    fn command(&self) -> &'static str {
        Self::COMMAND
    }
}

pub type CEBuildResult = Result<BinaryPacket, EventError>;
pub type CEParse = (Box<dyn ServerEvent>, Option<Vec<Box<dyn ServerEvent>>>);
pub type CEParseResult = Result<CEParse, EventError>;
pub type ParseEventFn = fn(Bytes, &Context) -> CEParseResult;

pub trait ClientEvent: CECommandMarker {
    fn packet_type(&self) -> PacketType {
        PacketType::T12 // most common packet type
    }
    fn build(&self, _: &Context) -> CEBuildResult;
    fn parse(packet: Bytes, context: &Context) -> CEParseResult;
}

pub struct ClientResult;

impl ClientResult {
    pub fn single(event: Box<dyn ServerEvent>) -> CEParse {
        (event, None)
    }

    pub fn with_extra(
        event: Box<dyn ServerEvent>,
        extra: Option<Vec<Box<dyn ServerEvent>>>,
    ) -> CEParse {
        (event, extra)
    }
}

pub struct ClientEventRegistry {
    pub command: &'static str,
    pub parse_fn: ParseEventFn,
}

inventory::collect!(ClientEventRegistry);

type EventMap = HashMap<&'static str, ParseEventFn>;
static EVENT_MAP: Lazy<EventMap> = Lazy::new(|| {
    let mut map = HashMap::new();
    for item in inventory::iter::<ClientEventRegistry> {
        map.insert(item.command, item.parse_fn);
    }
    map
});

pub async fn resolve_event(packet: SsoPacket, context: &Arc<Context>) -> CEParseResult {
    // Lagrange.Core.Internal.Context.ServiceContext.ResolveEventByPacket
    let payload = PacketReader::new(packet.payload()).section(|p| p.bytes());
    let Some(parse) = EVENT_MAP.get(packet.command()) else {
        return Err(EventError::UnsupportedEvent(packet.command().to_string()));
    };
    let events = parse(payload, context)?;
    Ok(events)
}

pub fn downcast_event<T: ServerEvent + 'static>(event: &impl AsRef<dyn ServerEvent>) -> Option<&T> {
    event.as_ref().as_any().downcast_ref::<T>()
}

pub fn downcast_major_event<T: ServerEvent + 'static>(event: &CEParse) -> Option<&T> {
    let (se, _) = event;
    se.as_any().downcast_ref::<T>()
}

pub fn downcast_mut_event<T: ServerEvent + 'static>(event: &mut dyn ServerEvent) -> Option<&mut T> {
    event.as_any_mut().downcast_mut::<T>()
}

pub fn downcast_mut_major_event<T: ServerEvent + 'static>(event: &mut CEParse) -> Option<&mut T> {
    let (se, _) = event;
    se.as_any_mut().downcast_mut::<T>()
}

#[derive(Debug, Error)]
pub enum EventError {
    #[error("unsupported event, commend: {0}")]
    UnsupportedEvent(String),

    #[error("TLV error occurred: {0}")]
    MissingTlv(#[from] crate::core::tlv::TlvError),

    #[error("failed to parse packet from raw proto in mania internal event: {0}")]
    ProtoParseError(#[from] prost::DecodeError),

    #[error("failed to parse mania internal packet: {0}")]
    PacketParseError(#[from] crate::core::packet::PacketError),

    #[error("An mania internal event error occurred: {0}")]
    OtherError(String),

    #[error("An internal oidb packet inner error occurred, ret_code: {0}, wording: {1}")]
    OidbPacketInternalError(i32, String),
}

pub(crate) mod prelude {
    pub use crate::core::context::Context;
    pub use crate::core::event::{
        CEBuildResult, CECommandMarker, CEParseResult, ClientEvent, ClientResult, EventError,
        ServerEvent,
    };
    pub use crate::core::packet::{
        BinaryPacket, OidbPacket, PREFIX_LENGTH_ONLY, PREFIX_U8, PREFIX_U16, PREFIX_WITH,
        PacketBuilder, PacketError, PacketReader, PacketType,
    };
    pub use crate::dda;
    pub use bytes::Bytes;
    pub use inventory;
    pub use mania_macros::{DummyEvent, ServerEvent, command, oidb_command};
    pub use num_enum::TryFromPrimitive;
    pub use prost::Message;
    pub use std::convert::TryFrom;
    pub use std::fmt::Debug;
}
