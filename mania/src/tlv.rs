use std::collections::HashMap;

use bytes::Bytes;
use phf::{phf_map, Map};
use thiserror::Error;

use crate::context::Context;
use crate::packet::{PacketBuilder, PacketReader};

pub mod t011;
pub mod t016;
pub mod t017;
pub mod t018;
pub mod t019;
pub mod t01b;
pub mod t01c;
pub mod t01d;
pub mod t01e;
pub mod t033;
pub mod t035;
pub mod t066;
pub mod t0d1;

type TlvConstructor = fn(&Context) -> Box<dyn TlvSer>;
static TLV_SER_MAP: Map<u16, TlvConstructor> = phf_map! {
    0x011_u16 => t011::T011::from_context,
    0x016_u16 => t016::T016::from_context,
    0x01b_u16 => t01b::T01b::from_context,
    0x01d_u16 => t01d::T01d::from_context,
    0x033_u16 => t033::T033::from_context,
    0x035_u16 => t035::T035::from_context,
    0x066_u16 => t066::T066::from_context,
    0x0d1_u16 => t0d1::T0d1::from_context,
};

type TlvDeserializer = fn(&mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError>;
static TLV_DE_MAP: Map<u16, TlvDeserializer> = phf_map! {
    0x017_u16 => t017::T017::deserialize,
    0x018_u16 => t018::T018::deserialize,
    0x019_u16 => t019::T019::deserialize,
    0x01c_u16 => t01c::T01c::deserialize,
    0x01e_u16 => t01e::T01e::deserialize,
    0x0d1_u16 => t0d1::T0d1Resp::deserialize,
};

pub trait TlvSer {
    fn from_context(ctx: &Context) -> Box<dyn TlvSer>
    where
        Self: Sized;

    fn serialize(&self, p: PacketBuilder) -> PacketBuilder;

    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.serialize(PacketBuilder::new()).build()
    }
}

/// Create a new TLV object by tag
pub fn new_tlv(tag: u16, ctx: &Context) -> Option<Box<dyn TlvSer>> {
    TLV_SER_MAP.get(&tag).map(|f| f(ctx))
}

pub fn serialize_tlv_set(ctx: &Context, tags: &[u16], mut packet: PacketBuilder) -> PacketBuilder {
    packet = packet.u16(tags.len() as u16);
    for &tag in tags {
        let tlv = new_tlv(tag, ctx).expect("tlv not found");
        packet = packet.bytes(tlv.serialize_to_bytes().as_slice());
    }
    packet
}

pub trait TlvDe {
    /// Deserialize a TLV object from a packet reader
    ///
    /// Tag is **not** included in the packet
    fn deserialize(reader: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError>
    where
        Self: Sized;

    fn tag(&self) -> u16;
    fn tag_static() -> u16
    where
        Self: Sized;

    fn as_any(&self) -> &dyn std::any::Any;
}

/// Deserialize a TLV object from a packet reader
pub fn deserialize_tlv(reader: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
    let tag = reader.u16();
    let de = TLV_DE_MAP
        .get(&tag)
        .ok_or(ParseTlvError::UnsupportedTag(tag))?;
    de(reader)
}

pub struct TlvSet(HashMap<u16, Box<dyn TlvDe>>);
impl TlvSet {
    pub fn deserialize(packet: Bytes) -> Self {
        let mut result = HashMap::new();

        let mut reader = PacketReader::new(packet);
        let count = reader.u16();

        for _ in 0..count {
            match deserialize_tlv(&mut reader) {
                Ok(tlv) => {
                    result.insert(tlv.tag(), tlv);
                }
                Err(e) => tracing::warn!("parse TLV error: {}", e),
            }
        }
        Self(result)
    }

    pub fn get<T: TlvDe + 'static>(&self) -> Result<&T, u16> {
        let tag = T::tag_static();
        self.0
            .get(&tag)
            .and_then(|tlv| tlv.as_any().downcast_ref::<T>())
            .ok_or(tag)
    }
}

#[derive(Debug, Error)]
pub enum ParseTlvError {
    #[error("unsupported TLV tag: 0x{0:04x}")]
    UnsupportedTag(u16),

    #[error("protobuf error: {0}")]
    ProtobufError(#[from] protobuf::Error),
}

mod prelude {
    pub use bytes::Bytes;

    pub use crate::context::Context;
    pub use crate::packet::{PacketBuilder, PacketReader};
    pub use crate::tlv::{ParseTlvError, TlvDe, TlvSer};

    impl PacketBuilder {
        pub(in crate::tlv) fn tlv(
            self,
            tag: u16,
            f: impl FnOnce(PacketBuilder) -> PacketBuilder,
        ) -> PacketBuilder {
            self.u16(tag).section_16_with_addition::<_, 0>(f)
        }

        pub(in crate::tlv) fn proto<T: protobuf::Message>(self, proto: &T) -> PacketBuilder {
            self.bytes(proto.write_to_bytes().unwrap().as_slice())
        }

        pub(in crate::tlv) fn bytes_with_length(self, bytes: &[u8]) -> PacketBuilder {
            self.section_16_with_addition::<_, 0>(|p| p.bytes(bytes))
        }

        pub(in crate::tlv) fn string_with_length(self, s: &str) -> PacketBuilder {
            self.bytes_with_length(s.as_bytes())
        }
    }

    impl PacketReader {
        pub(in crate::tlv) fn length_value<T>(
            &mut self,
            f: impl FnOnce(&mut PacketReader) -> T,
        ) -> T {
            self.section_16_with_addition::<_, 0>(f)
        }

        pub(in crate::tlv) fn proto<T: protobuf::Message>(&mut self) -> Result<T, ParseTlvError> {
            T::parse_from_bytes(&self.bytes()).map_err(ParseTlvError::ProtobufError)
        }

        pub(in crate::tlv) fn bytes_with_length(&mut self) -> Bytes {
            self.length_value(PacketReader::bytes)
        }

        pub(in crate::tlv) fn string_with_length(&mut self) -> String {
            String::from_utf8_lossy(&self.bytes_with_length()).into_owned()
        }
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! impl_tlv_de {
        ($tag:literal) => {
            fn tag(&self) -> u16 {
                $tag
            }

            fn tag_static() -> u16
            where
                Self: Sized,
            {
                $tag
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        };
    }
    pub use crate::impl_tlv_de;
}
