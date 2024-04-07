use crate::context::Context;
use crate::event::ClientEvent;
use crate::packet::{BinaryPacket, PacketType};

pub struct Alive;

impl ClientEvent for Alive {
    fn command(&self) -> &'static str {
        "Heartbeat.Alive"
    }

    fn packet_type(&self) -> PacketType {
        PacketType::T13
    }

    fn build_packets(&self, _ctx: &Context) -> Vec<BinaryPacket> {
        vec![BinaryPacket(4u32.to_be_bytes().to_vec().into())]
    }
}
