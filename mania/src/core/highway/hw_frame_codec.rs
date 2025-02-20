use crate::core::highway::HighwayError;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::Debug;
use tokio_util::codec::{Decoder, Encoder};

pub struct HighwayFrame {
    pub head: Bytes,
    pub body: Bytes,
}

impl Debug for HighwayFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HighwayFrame")
            .field("head", &hex::encode(self.head.clone()))
            .field("body", &hex::encode(self.body.clone()))
            .finish()
    }
}

pub struct HighwayFrameCodec;

impl Encoder<HighwayFrame> for HighwayFrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: HighwayFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u8(0x28);
        dst.put_u32(item.head.len() as u32);
        dst.put_u32(item.body.len() as u32);
        dst.put_slice(&item.head);
        dst.put_slice(&item.body);
        dst.put_u8(0x29);
        Ok(())
    }
}

impl Decoder for HighwayFrameCodec {
    type Item = HighwayFrame;
    type Error = HighwayError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 1 + 4 + 4 + 1 {
            tracing::trace!("Not enough data for frame (stage 1)!");
            return Ok(None);
        }
        let start = src.get_u8();
        let head_length = src.get_u32() as usize;
        let body_length = src.get_u32() as usize;
        if src.len() < head_length + body_length + 1 {
            tracing::trace!("Not enough data for frame (stage 2)!");
            return Ok(None);
        }
        let head = src.split_to(head_length);
        let body = src.split_to(body_length);
        let end = src.get_u8();
        if start != 0x28 || end != 0x29 {
            return Err(HighwayError::InvalidFrame);
        }
        Ok(Some(Self::Item {
            head: head.into(),
            body: body.into(),
        }))
    }
}
