use bytes::Bytes;
use phf::{phf_map, Map};
use thiserror::Error;

use crate::context::Context;
use crate::packet::{BinaryPacket, PacketReader, PacketType, SsoPacket};

pub mod alive;
pub mod trans_emp;

pub trait ClientEvent: Send + Sync {
    fn command(&self) -> &'static str;
    fn packet_type(&self) -> PacketType;
    fn build_packets(&self, context: &Context) -> Vec<BinaryPacket>;

    fn build_sso_packets(&self, context: &Context) -> Vec<SsoPacket> {
        let packet_type = self.packet_type();
        let command = self.command();
        self.build_packets(context)
            .into_iter()
            .map(|packet| SsoPacket::new(packet_type, command, packet))
            .collect()
    }
}

pub trait ServerEvent: std::fmt::Debug + Send + Sync {
    fn ret_code(&self) -> i32;

    fn as_any(&self) -> &dyn std::any::Any;
}

type ParseEvent = fn(Bytes, &Context) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError>;
static EVENT_MAP: Map<&'static str, ParseEvent> = phf_map! {
    "wtlogin.trans_emp" => trans_emp::parse,
};

/// Resolve SSO events from a packet.
pub fn resolve_events(
    packet: SsoPacket,
    context: &Context,
) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError> {
    // Lagrange.Core.Internal.Context.ServiceContext.ResolveEventByPacket
    let payload = PacketReader::new(packet.payload()).section(|p| p.bytes());

    let Some(parse) = EVENT_MAP.get(packet.command()) else {
        tracing::warn!("unsupport SSO event: {}", packet.command());
        tracing::debug!("payload: {:?}", payload);
        return Err(ParseEventError::UnsupportedEvent(
            packet.command().to_string(),
        ));
    };

    let events = parse(payload, context)?;
    tracing::debug!("receive SSO event: {}", packet.command());
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
