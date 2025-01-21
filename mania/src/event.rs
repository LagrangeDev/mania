pub mod alive;
pub mod info_sync;
pub mod trans_emp;
pub mod wtlogin;
// TODO: global mod(s)

use crate::context::Context;
use crate::event::alive::Alive;
use crate::event::info_sync::InfoSync;
use crate::event::wtlogin::WtLogin;
use crate::packet::{BinaryPacket, PacketReader, PacketType, SsoPacket};
use crate::TransEmp;
use bytes::Bytes;
use phf::phf_map;
use std::sync::Arc;
use thiserror::Error;

pub trait ServerEvent: std::fmt::Debug + Send + Sync {
    fn ret_code(&self) -> i32;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait ClientEvent: Send + Sync {
    const COMMAND: &'static str;
    fn command(&self) -> &'static str {
        Self::COMMAND
    }
    fn packet_type(&self) -> PacketType {
        PacketType::T12 // most common packet type
    }
    fn build(&self, context: &Context) -> Vec<BinaryPacket>;
    fn build_sso_packets(&self, context: &Context) -> Vec<SsoPacket> {
        let packet_type = self.packet_type();
        let command = self.command();
        self.build(context)
            .into_iter()
            .map(|packet| SsoPacket::new(packet_type, command, packet))
            .collect()
    }
    fn parse(
        packet: Bytes,
        context: &Context,
    ) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError>;
}

type ParseEvent = fn(Bytes, &Context) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError>;
static EVENT_MAP: phf::Map<&'static str, ParseEvent> = phf_map! {
    "wtlogin.trans_emp" => TransEmp::parse,
    "wtlogin.login" => WtLogin::parse,
    "Heartbeat.Alive" => Alive::parse,
    "trpc.msg.register_proxy.RegisterProxy.SsoInfoSync" => InfoSync::parse,
};

/// Resolve SSO events from a packet.
pub async fn resolve_events(
    packet: SsoPacket,
    context: &Arc<Context>,
) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError> {
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
