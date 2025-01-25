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

pub trait ClientEvent: CECommandMarker {
    fn packet_type(&self) -> PacketType {
        PacketType::T12 // most common packet type
    }
    fn build(&self, context: &Context) -> BinaryPacket;
    fn parse(packet: Bytes, context: &Context) -> Result<Box<dyn ServerEvent>, ParseEventError>;
}

type ParseEvent = fn(Bytes, &Context) -> Result<Box<dyn ServerEvent>, ParseEventError>;

pub struct ClientEventRegistry {
    pub command: &'static str,
    pub parse_fn: ParseEvent,
}

inventory::collect!(ClientEventRegistry);

type EventMap = HashMap<&'static str, ParseEvent>;
static EVENT_MAP: Lazy<EventMap> = Lazy::new(|| {
    let mut map = HashMap::new();
    for item in inventory::iter::<ClientEventRegistry> {
        map.insert(item.command, item.parse_fn);
    }
    map
});

pub async fn resolve_event(
    packet: SsoPacket,
    context: &Arc<Context>,
) -> Result<Box<dyn ServerEvent>, ParseEventError> {
    // Lagrange.Core.Internal.Context.ServiceContext.ResolveEventByPacket
    let payload = PacketReader::new(packet.payload()).section(|p| p.bytes());
    tracing::debug!(
        "receive SSO event: {}, payload: {:?}",
        packet.command(),
        hex::encode(&payload)
    );
    let Some(parse) = EVENT_MAP.get(packet.command()) else {
        return Err(ParseEventError::UnsupportedEvent(
            packet.command().to_string(),
        ));
    };
    let events = parse(payload, context)?;
    Ok(events)
}

pub fn downcast_event<T: ServerEvent + 'static>(event: &impl AsRef<dyn ServerEvent>) -> Option<&T> {
    event.as_ref().as_any().downcast_ref::<T>()
}

#[derive(Debug, Error)]
pub enum ParseEventError {
    #[error("unsupported event: {0}")]
    UnsupportedEvent(String),

    #[error("invalid packet header")]
    InvalidPacketHeader,

    #[error("invalid packet end")]
    InvalidPacketEnd,

    #[error("unsupported trans_emp command: {0}")]
    UnsupportedTransEmp(u16),

    #[error("missing or corrupted TLV: {0}")]
    MissingTlv(u16),

    #[error("unknown ret code: {0}")]
    UnknownRetCode(i32),
}

pub(crate) mod prelude {
    pub use crate::core::context::Context;
    pub use crate::core::event::{
        CECommandMarker, ClientEvent, ClientEventRegistry, ParseEventError, ServerEvent,
    };
    pub use crate::core::packet::{
        BinaryPacket, PacketBuilder, PacketReader, PacketType, PREFIX_LENGTH_ONLY, PREFIX_U16,
        PREFIX_U8, PREFIX_WITH,
    };
    pub use bytes::Bytes;
    pub use inventory;
    pub use mania_macros::{ce_commend, ServerEvent};
    pub use protobuf::{Message as ProtoMessage, MessageField as ProtoMessageField};
    pub use std::{collections::HashMap, fmt::Debug};
}
