pub mod alive;
pub mod info_sync;
pub mod trans_emp;
pub mod wtlogin;

use crate::context::Context;
use crate::packet::{BinaryPacket, PacketReader, PacketType, SsoPacket};
use bytes::Bytes;
use once_cell::sync::Lazy;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use thiserror::Error;

#[derive(Copy, Clone, Debug)]
pub enum LogicFlow {
    InComing,
    OutGoing,
}

impl Display for LogicFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicFlow::InComing => write!(f, ">>> InComing"),
            LogicFlow::OutGoing => write!(f, "<<< OutGoing"),
        }
    }
}

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

type LogicHandleFn = fn(&mut dyn ServerEvent, LogicFlow) -> &dyn ServerEvent;
pub struct LogicRegistry {
    pub event_type_id_fn: fn() -> TypeId,
    pub event_handle_fn: LogicHandleFn,
}

inventory::collect!(LogicRegistry);

type LogicHandlerMap = HashMap<TypeId, Vec<LogicHandleFn>>;
static LOGIC_MAP: Lazy<LogicHandlerMap> = Lazy::new(|| {
    let mut map = HashMap::new();
    for item in inventory::iter::<LogicRegistry> {
        let tid = (item.event_type_id_fn)();
        map.entry(tid)
            .or_insert_with(Vec::new)
            .push(item.event_handle_fn);
        tracing::debug!(
            "Registered event handler {:?} for type: {:?}",
            item.event_handle_fn,
            tid
        );
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

pub fn dispatch_logic(event: &mut dyn ServerEvent, flow: LogicFlow) -> &dyn ServerEvent {
    let tid = event.as_any().type_id();
    if let Some(fns) = LOGIC_MAP.get(&tid) {
        tracing::debug!("[{}] Found {} handlers for {:?}.", flow, fns.len(), event);
        for handle_fn in fns.iter() {
            handle_fn(event, flow);
        }
    } else {
        tracing::debug!("[{}] No handler found for {:?}", flow, event);
    }
    event
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

mod prelude {
    pub use crate::context::Context;
    pub use crate::event::{
        CECommandMarker, ClientEvent, ClientEventRegistry, ParseEventError, ServerEvent,
    };
    pub use crate::packet::{
        BinaryPacket, PacketBuilder, PacketReader, PacketType, PREFIX_LENGTH_ONLY, PREFIX_U16,
        PREFIX_U8, PREFIX_WITH,
    };
    pub use bytes::Bytes;
    pub use inventory;
    pub use mania_macros::{ce_commend, ServerEvent};
    pub use protobuf::{Message as ProtoMessage, MessageField as ProtoMessageField};
    pub use std::{collections::HashMap, fmt::Debug};
}
