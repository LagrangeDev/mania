use crate::context::Context;
use crate::event::*;
use crate::packet::{BinaryPacket, PacketType};
use bytes::Bytes;
use mania_macros::ce_commend;

#[ce_commend("Heartbeat.Alive")]
pub struct Alive;

#[derive(Debug)]
pub struct AliveRes;

impl ServerEvent for AliveRes {
    fn ret_code(&self) -> i32 {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ClientEvent for Alive {
    fn packet_type(&self) -> PacketType {
        PacketType::T13
    }

    fn build(&self, _ctx: &Context) -> Vec<BinaryPacket> {
        vec![BinaryPacket(4u32.to_be_bytes().to_vec().into())]
    }

    fn parse(_: Bytes, _: &Context) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError> {
        Ok(vec![Box::new(AliveRes {})])
    }
}
