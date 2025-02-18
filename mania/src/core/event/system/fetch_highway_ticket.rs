use crate::core::event::prelude::*;
use crate::core::protos::action::{HttpConn, HttpConn0x6ff501, HttpConn0x6ff501response};

#[command("HttpConn.0x6ff_501")]
#[derive(Debug, ServerEvent, Default)]
pub struct FetchHighwayTicketEvent {
    pub sig_session: Bytes,
}

impl ClientEvent for FetchHighwayTicketEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let packet = HttpConn0x6ff501 {
            http_conn: Some(dda!(HttpConn {
                field1: 0,
                field2: 0,
                field3: 16,
                field4: 1,
                field6: 3,
                service_types: vec![1, 5, 10, 21],
                field9: 2,
                field10: 9,
                field11: 8,
                ver: "1.0.1".to_string(),
            })),
        };
        Ok(BinaryPacket(packet.encode_to_vec().into()))
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let res = HttpConn0x6ff501response::decode(packet)?;
        let res = res
            .http_conn
            .ok_or_else(|| EventError::OtherError("No http_conn in response".to_string()))?;
        Ok(ClientResult::single(Box::new(FetchHighwayTicketEvent {
            sig_session: Bytes::from(res.sig_session),
        })))
    }
}
