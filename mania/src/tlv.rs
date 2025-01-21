use std::collections::HashMap;

use bytes::Bytes;
use phf::{phf_map, Map};
use thiserror::Error;

use crate::context::Context;
use crate::packet::{PacketBuilder, PacketReader};

pub mod t011q;
pub mod t016a;
pub mod t016e;
pub mod t016q;
pub mod t017q;
pub mod t018;
pub mod t018q;
pub mod t019q;
pub mod t01bq;
pub mod t01cq;
pub mod t01dq;
pub mod t01eq;
pub mod t033q;
pub mod t035q;
pub mod t066q;
pub mod t0d1q;
pub mod t100;
pub mod t106;
pub mod t107;
pub mod t10a;
pub mod t116;
pub mod t119;
pub mod t11a;
pub mod t124;
pub mod t128;
pub mod t141;
pub mod t142;
pub mod t143;
pub mod t144;
pub mod t145;
pub mod t146;
pub mod t147;
pub mod t166;
pub mod t177;
pub mod t191;
pub mod t305;
pub mod t318;
pub mod t521;
pub mod t543;

pub struct TlvPreload {
    unusual_sign: Option<Bytes>,
    no_pic_sig: Option<Bytes>,
    uin: u32,
    tgtgt_key: [u8; 16],
    temp_password: Option<Bytes>,
}

impl TlvPreload {
    pub fn new(
        unusual_sign: Option<Bytes>,
        no_pic_sig: Option<Bytes>,
        uin: u32,
        tgtgt_key: [u8; 16],
        temp_password: Option<Bytes>,
    ) -> Self {
        Self {
            unusual_sign,
            no_pic_sig,
            uin,
            tgtgt_key,
            temp_password,
        }
    }
}

type TlvConstructor = fn(&Context) -> Box<dyn TlvSer>;
static TLV_QR_SER_MAP: Map<u16, TlvConstructor> = phf_map! {
    0x011_u16 => t011q::T011q::from_context,
    0x016_u16 => t016q::T016q::from_context,
    0x01b_u16 => t01bq::T01bq::from_context,
    0x01d_u16 => t01dq::T01dq::from_context,
    0x033_u16 => t033q::T033q::from_context,
    0x035_u16 => t035q::T035q::from_context,
    0x066_u16 => t066q::T066q::from_context,
    0x0d1_u16 => t0d1q::T0d1q::from_context,
};

static TLV_SER_MAP: Map<u16, TlvConstructor> = phf_map! {
    0x106_u16 => t106::T106::from_context,
    0x144_u16 => t144::T144::from_context,
    0x116_u16 => t116::T116::from_context,
    0x142_u16 => t142::T142::from_context,
    0x145_u16 => t145::T145::from_context,
    0x018_u16 => t018::T018::from_context,
    0x141_u16 => t141::T141::from_context,
    0x177_u16 => t177::T177::from_context,
    0x191_u16 => t191::T191::from_context,
    0x100_u16 => t100::T100::from_context,
    0x107_u16 => t107::T107::from_context,
    0x318_u16 => t318::T318::from_context,
    0x16a_u16 => t016a::T16A::from_context,
    0x166_u16 => t166::T166::from_context,
    0x521_u16 => t521::T521::from_context,
    0x16E_u16 => t016e::T016E::from_context,
    0x147_u16 => t147::T147::from_context,
    0x128_u16 => t128::T128::from_context,
    0x124_u16 => t124::T124::from_context,
};

type TlvDeserializer = fn(&mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError>;
static TLV_QR_DE_MAP: Map<u16, TlvDeserializer> = phf_map! {
    0x017_u16 => t017q::T017q::deserialize,
    0x018_u16 => t018q::T018q::deserialize,
    0x019_u16 => t019q::T019q::deserialize,
    0x01c_u16 => t01cq::T01cq::deserialize,
    0x01e_u16 => t01eq::T01eq::deserialize,
    0x0d1_u16 => t0d1q::T0d1Resp::deserialize,
};

static TLV_DE_MAP: Map<u16, TlvDeserializer> = phf_map! {
    0x106_u16 => t106::T106::deserialize,
    0x10A_u16 => t10a::T10A::deserialize,
    0x119_u16 => t119::T119::deserialize,
    0x143_u16 => t143::T143::deserialize,
    0x146_u16 => t146::T146::deserialize,
    0x305_u16 => t305::T305::deserialize,
    0x543_u16 => t543::T543::deserialize,
    0x11A_u16 => t11a::T11A::deserialize,
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
pub fn new_qrcode_tlv(tag: u16, ctx: &Context) -> Option<Box<dyn TlvSer>> {
    TLV_QR_SER_MAP.get(&tag).map(|f| f(ctx))
}

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

pub fn serialize_qrcode_tlv_set(
    ctx: &Context,
    tags: &[u16],
    mut packet: PacketBuilder,
) -> PacketBuilder {
    packet = packet.u16(tags.len() as u16);
    for &tag in tags {
        let tlv = new_qrcode_tlv(tag, ctx).expect("tlv not found");
        packet = packet.bytes(tlv.serialize_to_bytes().as_slice());
    }
    packet
}

pub trait TlvDe: Send {
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
pub fn deserialize_qrcode_tlv(reader: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
    let tag = reader.u16();
    let de = TLV_QR_DE_MAP.get(&tag).ok_or_else(|| {
        let len = reader.u16();
        reader.read_packet(len as usize);
        ParseTlvError::UnsupportedTag(tag)
    })?;
    de(reader)
}

pub fn deserialize_tlv(reader: &mut PacketReader) -> Result<Box<dyn TlvDe>, ParseTlvError> {
    let tag = reader.u16();
    let de = TLV_DE_MAP.get(&tag).ok_or_else(|| {
        let len = reader.u16();
        reader.read_packet(len as usize);
        ParseTlvError::UnsupportedTag(tag)
    })?;
    de(reader)
}

pub struct TlvSet(HashMap<u16, Box<dyn TlvDe>>);
impl TlvSet {
    pub fn deserialize_qrcode(packet: Bytes) -> Self {
        let mut result = HashMap::new();

        let mut reader = PacketReader::new(packet);
        let count = reader.u16();

        for _ in 0..count {
            match deserialize_qrcode_tlv(&mut reader) {
                Ok(tlv) => {
                    result.insert(tlv.tag(), tlv);
                }
                Err(e) => tracing::warn!("parse TLV error: {}", e),
            }
        }
        Self(result)
    }

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
    pub use crate::context::Context;
    pub use crate::context::ExtendUuid;
    pub use crate::crypto::tea::tea_encrypt;
    pub use crate::packet::{PacketBuilder, PacketReader};
    pub use crate::tlv::{serialize_tlv_set, ParseTlvError, TlvDe, TlvSer};
    pub use bytes::Bytes;
    pub use uuid::Uuid;

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

        // Prefix.Uint16 | Prefix.LengthOnly
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
