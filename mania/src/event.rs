pub mod alive;
pub mod info_sync;
pub mod trans_emp;
pub mod wtlogin;
// TODO: global mod(s)

use crate::context::Context;
use crate::packet::{BinaryPacket, PacketReader, PacketType, SsoPacket};
use bytes::Bytes;
pub use inventory;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

pub trait ServerEvent: Debug + Send + Sync {
    fn ret_code(&self) -> i32;
    fn as_any(&self) -> &dyn std::any::Any;
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
    fn parse(
        packet: Bytes,
        context: &Context,
    ) -> Result<Box<dyn ServerEvent>, ParseEventError>;
}

pub fn build_sso_packet<T: ClientEvent>(event: &T, context: &Context) -> SsoPacket {
    SsoPacket::new(event.packet_type(), event.command(), event.build(context))
}

type ParseEvent = fn(Bytes, &Context) -> Result<Box<dyn ServerEvent>, ParseEventError>;

pub struct ClientEventRegistry {
    pub command: &'static str,
    pub parse_fn: ParseEvent,
}

inventory::collect!(ClientEventRegistry);

type EventMapT = HashMap<&'static str, ParseEvent>;
static EVENT_MAP: Lazy<EventMapT> = Lazy::new(|| {
    let mut map = HashMap::new();
    for item in inventory::iter::<ClientEventRegistry> {
        map.insert(item.command, item.parse_fn);
    }
    map
});

/// Resolve SSO events from a packet.
pub async fn resolve_events(
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

/// Downcast a protocol event to a specific type.
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
