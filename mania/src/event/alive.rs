use crate::context::Context;
use crate::event::{ClientEvent, ParseEventError, ServerEvent};
use crate::packet::{BinaryPacket, PacketType};
use bytes::Bytes;

pub struct Alive;

impl ClientEvent for Alive {
    fn command(&self) -> &'static str {
        "Heartbeat.Alive"
    }

    fn packet_type(&self) -> PacketType {
        PacketType::T13
    }

    async fn build_packets(&self, _ctx: &Context) -> Vec<BinaryPacket> {
        vec![BinaryPacket(4u32.to_be_bytes().to_vec().into())]
    }
}

#[derive(Debug)]
pub struct AliveRes {}

impl ServerEvent for AliveRes {
    fn ret_code(&self) -> i32 {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn parse(_: Bytes, _: &Context) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError> {
    Ok(vec![Box::new(AliveRes {})])
}
