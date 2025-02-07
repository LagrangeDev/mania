use std::borrow::Cow;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::sync::atomic::AtomicU32;

use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use prost::Message;
use rand::Rng;
use thiserror::Error;

use crate::core::context::Context;
use crate::core::crypto::tea;
use crate::core::protos::service::oidb::{OidbLafter, OidbSvcTrpcTcpBase};
use crate::core::protos::system::{NtDeviceSign, NtPacketUid, Sign};
use crate::dda;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PacketType {
    T12 = 12,
    T13 = 13,
}

impl TryFrom<u32> for PacketType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            12 => Ok(PacketType::T12),
            13 => Ok(PacketType::T13),
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
pub struct BinaryPacket(pub Bytes);

pub struct OidbPacket {
    commend: u32,
    sub_commend: u32,
    body: Bytes,
    is_lafter: bool,
    is_uid: bool, // aka reserved
}

impl OidbPacket {
    pub fn new(
        commend: u32,
        sub_commend: u32,
        body: Vec<u8>,
        is_lafter: bool,
        is_uid: bool,
    ) -> Self {
        Self {
            commend,
            sub_commend,
            body: Bytes::from(body),
            is_lafter,
            is_uid,
        }
    }

    pub fn to_binary(&self) -> BinaryPacket {
        let base = dda!(OidbSvcTrpcTcpBase {
            command: self.commend,
            sub_command: self.sub_commend,
            body: self.body.to_vec(),
            reserved: i32::from(self.is_uid),
            lafter: if self.is_lafter {
                Some(OidbLafter::default())
            } else {
                None
            },
        });
        BinaryPacket(Bytes::from(base.encode_to_vec()))
    }

    pub fn parse(packet: Bytes) -> Result<Self, PacketError> {
        match OidbSvcTrpcTcpBase::decode(packet) {
            Ok(base) => match base.error_code {
                0 => Ok(Self {
                    commend: base.command,
                    sub_commend: base.sub_command,
                    body: Bytes::from(base.body),
                    is_lafter: base.lafter.is_some(),
                    is_uid: base.reserved != 0,
                }),
                code => Err(PacketError::OidbPacketRetFailed {
                    command: format!("0x{:x}_{}", base.command, base.sub_command),
                    ret_code: code,
                    client_wording: base.error_msg,
                }),
            },
            Err(e) => Err(PacketError::SsoPacketParseFailed(e)),
        }
    }

    pub fn parse_into<T>(packet: Bytes) -> Result<T, PacketError>
    where
        T: prost::Message + Default,
    {
        Ok(Self::parse(packet)?.into_body::<T>()?)
    }

    fn into_body<T>(self) -> Result<T, prost::DecodeError>
    where
        T: prost::Message + Default,
    {
        T::decode(self.body)
    }
}

pub struct SsoPacket {
    packet_type: PacketType,
    command: Cow<'static, str>,
    sequence: u32,
    payload: BinaryPacket,
}

impl Debug for SsoPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SsoPacket")
            .field("packet_type", &self.packet_type)
            .field("command", &self.command)
            .field("sequence", &self.sequence)
            .field("payload", &hex::encode(&self.payload.0))
            .finish()
    }
}

// TODO: add tests
impl SsoPacket {
    pub fn new(
        packet_type: PacketType,
        command: impl Into<Cow<'static, str>>,
        payload: BinaryPacket,
    ) -> Self {
        SsoPacket {
            packet_type,
            command: command.into(),
            sequence: Self::next_sequence(),
            payload,
        }
    }

    pub fn with_sequence(
        packet_type: PacketType,
        command: impl Into<Cow<'static, str>>,
        sequence: u32,
        payload: BinaryPacket,
    ) -> Self {
        SsoPacket {
            packet_type,
            command: command.into(),
            sequence,
            payload,
        }
    }

    pub fn next_sequence() -> u32 {
        use std::sync::atomic::Ordering::*;
        static SEQUENCE: AtomicU32 = AtomicU32::new(0);

        // Initialize the sequence number
        if SEQUENCE.compare_exchange(0, 1, Release, Acquire).is_ok() {
            let offset = rand::thread_rng().gen_range(5000000..=9900000);
            SEQUENCE.store(offset, Relaxed);
        } else {
            // Other threads are doing the initialization
            while SEQUENCE.load(Relaxed) == 1 {
                std::thread::yield_now();
            }
        }

        SEQUENCE.fetch_add(1, Relaxed)
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn sequence(&self) -> u32 {
        self.sequence
    }

    pub fn payload(&self) -> Bytes {
        self.payload.0.clone()
    }

    pub fn build(&self, ctx: &Context) -> Vec<u8> {
        match self.packet_type {
            PacketType::T12 => self.build_protocol12(ctx),
            PacketType::T13 => self.build_protocol13(ctx),
        }
    }

    pub fn build_protocol12(&self, ctx: &Context) -> Vec<u8> {
        // Lagrange.Core.Internal.Packets.SsoPacker.Build
        let body = PacketBuilder::new()
            .section(|header| {
                header
                    .u32(self.sequence) // sequence
                    .u32(ctx.app_info.sub_app_id as u32) // app id
                    .u32(2052) // locale id
                    .bytes(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                    .section(|p| p.bytes(&ctx.key_store.session.tgt.load())) // tgt
                    .section(|p| p.string(&self.command)) // command
                    .section(|p| p) // unknown
                    .section(|p| p.string(&hex::encode(ctx.device.uuid))) // uuid
                    .section(|p| p) // unknown
                    .section_16(|p| p.string(ctx.app_info.current_version)) // version
                    .section(|signature| {
                        let sign =
                            ctx.sign_provider
                                .sign(&self.command, self.sequence, &self.payload.0);
                        let device_sign = NtDeviceSign {
                            trace: random_trace(),
                            uid: ctx
                                .key_store
                                .uid
                                .load()
                                .as_ref()
                                .map(|arc| arc.as_ref().clone()),
                            sign: match sign {
                                Some(sign) => Some(Sign {
                                    sec_sign: Some(hex::decode(sign.sign).unwrap()),
                                    sec_extra: Some(hex::decode(sign.extra).unwrap()),
                                    sec_token: Some(sign.token),
                                }),
                                None => None,
                            },
                        };
                        let device_sign = device_sign.encode_to_vec();
                        signature.bytes(&device_sign)
                    }) // signature
            })
            .section(|payload| payload.bytes(&self.payload.0))
            .build();
        // Lagrange.Core.Internal.Packets.ServicePacker.BuildProtocol12
        PacketBuilder::new()
            .section(|packet| {
                packet
                    .u32(12) // protocol version
                    .u8(if ctx.key_store.session.d2.load().is_empty() {
                        2
                    } else {
                        1
                    }) // flag
                    .section(|p| p.bytes(&ctx.key_store.session.d2.load()))
                    .u8(0) // unknown
                    .section(|packet| packet.string(&ctx.key_store.uin.to_string())) // uin
                    .bytes(&tea::tea_encrypt(
                        &body,
                        ctx.key_store.session.d2_key.load().as_ref(),
                    ))
            })
            .build()
    }

    pub fn build_protocol13(&self, ctx: &Context) -> Vec<u8> {
        // Lagrange.Core.Internal.Packets.ServicePacker.BuildProtocol13
        PacketBuilder::new()
            .section(|packet| {
                packet
                    .u32(13) // protocol version
                    .u8(0) // flag
                    .u32(self.sequence) // sequence
                    .u8(0) // unknown
                    .section(|p| p.string("0")) // unknown
                    .section(|p| {
                        p.section(|p| p.string(&self.command)) // command
                            .section(|p| p) // unknown
                            .section(|p| {
                                p.bytes(&if let Some(uid) = ctx.key_store.uid.load().as_deref() {
                                    let uid = NtPacketUid {
                                        uid: Some(uid.clone()),
                                    };
                                    uid.encode_to_vec()
                                } else {
                                    Vec::new()
                                })
                            }) // uid
                    })
                    .section(|p| p.bytes(&self.payload.0)) // payload
            })
            .build()
    }

    pub fn parse(packet: Bytes, ctx: &Context) -> Result<SsoPacket, PacketError> {
        // Lagrange.Core.Internal.Packets.ServicePacker.Parse
        let mut reader = PacketReader::new(packet);
        let _length = reader.u32();
        let protocol: PacketType = reader
            .u32()
            .try_into()
            .map_err(PacketError::UnknownSsoPacketType)?;

        let auth_flag = reader.u8();
        let _flag = reader.u8();
        let uin = reader.section(|p| p.string());
        let uin = uin.parse().map_err(|_| PacketError::InvalidUin(uin))?;
        if uin != **ctx.key_store.uin.load() && protocol == PacketType::T12 {
            return Err(PacketError::UinMismatch(**ctx.key_store.uin.load(), uin));
        }

        let body = reader.bytes();

        let body = match auth_flag {
            0 => body,
            1 => tea::tea_decrypt(&body, ctx.key_store.session.d2_key.load().as_ref()).into(),
            2 => tea::tea_decrypt(&body, &[0u8; 16]).into(),
            _ => {
                return Err(PacketError::UnknownAuthFlag(auth_flag));
            }
        };

        // Lagrange.Core.Internal.Packets.SsoPacker.Parse
        let mut reader = PacketReader::new(body);
        let _length = reader.u32();
        let sequence = reader.u32();
        let ret_code = reader.i32();
        let extra = reader.section(|p| p.string());
        let command = reader.section(|p| p.string());
        let _ = reader.section(|p| p.bytes()); // unknown
        let is_compressed = reader.u32() != 0;
        reader.read_with_length::<_, { PREFIX_U32 | PREFIX_LENGTH_ONLY }>(|p| p.bytes()); // dummy sso header
        let body = if is_compressed {
            // Lagrange.Core.Internal.Packets.SsoPacker.InflatePacket
            let body = reader.section(|p| p.bytes());
            let mut reader = flate2::read::ZlibDecoder::new(body.reader());

            let mut buffer = BytesMut::new().writer();
            buffer.write_u32::<BigEndian>(0)?; // placeholder for length
            std::io::copy(&mut reader, &mut buffer)?;

            let mut buffer = buffer.into_inner();
            let len = buffer.len() as u32;
            BigEndian::write_u32(&mut buffer[0..4], len);

            buffer.freeze()
        } else {
            reader.bytes() // ...or full?
        };

        if ret_code == 0 {
            Ok(SsoPacket::with_sequence(
                PacketType::T12,
                command,
                sequence,
                BinaryPacket(body),
            ))
        } else {
            Err(PacketError::SsoPacketRetFailed {
                command,
                sequence,
                ret_code,
                extra,
            })
        }
    }
}

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("unknown raw SsoPacket type: {0}")]
    UnknownSsoPacketType(u32),

    #[error("unknown raw SsoPacket auth flag: {0}")]
    UnknownAuthFlag(u8),

    #[error("invalid uin: {0}")]
    InvalidUin(String),

    #[error("uin mismatch, expected {0}, got {1}")]
    UinMismatch(u32, u32),

    #[error("failed to inflate SsoPacket")]
    InflateFailed(#[from] std::io::Error),

    #[error("failed to parse SsoPacket from raw proto: {0}")]
    SsoPacketParseFailed(#[from] prost::DecodeError),

    #[error("SsoPacket {command} #{sequence} ret failed with code {ret_code}, extra: {extra}")]
    SsoPacketRetFailed {
        command: String,
        sequence: u32,
        ret_code: i32,
        extra: String,
    },

    #[error(
        "OidbPacket {command} ret failed with code {ret_code}, client_wording: {client_wording}"
    )]
    OidbPacketRetFailed {
        command: String,
        ret_code: u32,
        client_wording: String,
    },

    #[error("An error occurred when parsing packet: {0}")]
    OtherError(String),
}

fn random_trace() -> String {
    use std::fmt::Write;
    let mut result = String::from("00-");
    let mut rng = rand::thread_rng();

    // 32 digits
    for _ in 0..16 {
        write!(result, "{:x}", rng.r#gen::<u8>()).unwrap();
    }
    result.push('-');
    // 16 digits
    for _ in 0..8 {
        write!(result, "{:x}", rng.r#gen::<u8>()).unwrap();
    }
    result.push_str("-01");

    result
}

pub const PREFIX_NONE: u8 = 0b0000;
pub const PREFIX_U8: u8 = 0b0001;
pub const PREFIX_U16: u8 = 0b0010;
pub const PREFIX_U32: u8 = 0b0100;
pub const PREFIX_LENGTH_ONLY: u8 = 0;
pub const PREFIX_WITH: u8 = 0b1000;

pub struct PacketBuilder {
    buffer: Vec<u8>,
}

impl PacketBuilder {
    pub fn new() -> Self {
        PacketBuilder { buffer: Vec::new() }
    }

    pub fn u8(mut self, value: u8) -> Self {
        self.buffer.push(value);
        self
    }

    pub fn u16(mut self, value: u16) -> Self {
        self.buffer.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u32(mut self, value: u32) -> Self {
        self.buffer.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u64(mut self, value: u64) -> Self {
        self.buffer.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn i8(self, value: i8) -> Self {
        self.u8(value as u8)
    }

    pub fn i16(self, value: i16) -> Self {
        self.u16(value as u16)
    }

    pub fn i32(self, value: i32) -> Self {
        self.u32(value as u32)
    }

    pub fn i64(self, value: i64) -> Self {
        self.u64(value as u64)
    }

    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.buffer.extend_from_slice(bytes);
        self
    }

    pub fn string(self, string: &str) -> Self {
        self.bytes(string.as_bytes())
    }

    pub fn packet(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }

    pub fn write_with_length<F, const P: u8, const A: isize>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        const PREFIX_WITH_LENGTH: u8 = 0b1000;
        let prefix_length = (P & 0b0111) as usize;

        if prefix_length == 0 {
            return f(self);
        }

        let this = match prefix_length {
            1 => self.u8(0),
            2 => self.u16(0),
            4 => self.u32(0),
            _ => panic!("Invalid Prefix is given"),
        };

        let ori_length = this.buffer.len();

        let mut this = f(this);

        let closure_called_length = this.buffer.len() - ori_length;

        let length = (if (P & PREFIX_WITH_LENGTH) > 0 {
            closure_called_length + prefix_length
        } else {
            closure_called_length
        } as isize)
            + A;

        match prefix_length {
            1 => this.buffer[ori_length - prefix_length] = length as u8,
            2 => byteorder::BigEndian::write_u16(
                &mut this.buffer[ori_length - prefix_length..],
                length as u16,
            ),
            4 => byteorder::BigEndian::write_u32(
                &mut this.buffer[ori_length - prefix_length..],
                length as u32,
            ),
            _ => panic!("Invalid Prefix is given"),
        }

        this
    }

    // u32 | prefix
    pub fn section<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self.section_with_addition::<F, 4>(f)
    }

    // u32 + skip (no prefix)
    pub fn section_with_addition<F, const N: isize>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let start = self.buffer.len();
        let this = self.u32(0);

        let mut this = f(this);

        let len = (this.buffer.len() - start) as isize - 4 + N;
        byteorder::BigEndian::write_u32(&mut this.buffer[start..], len as u32);

        this
    }

    // u16 | prefix
    pub fn section_16<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self.section_16_with_addition::<F, 2>(f)
    }

    // u16 + skip (no prefix)
    pub fn section_16_with_addition<F, const N: isize>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let start = self.buffer.len();
        let this = self.u16(0);

        let mut this = f(this);

        let len = (this.buffer.len() - start) as isize - 2 + N;
        if len > u16::MAX as isize {
            panic!("section too long");
        }
        byteorder::BigEndian::write_u16(&mut this.buffer[start..], len as u16);

        this
    }

    pub fn build(self) -> Vec<u8> {
        self.buffer
    }
}

pub struct PacketReader {
    reader: Bytes,
}

impl PacketReader {
    pub fn new(packet: Bytes) -> Self {
        PacketReader { reader: packet }
    }

    pub fn u8(&mut self) -> u8 {
        self.reader.get_u8()
    }

    pub fn u16(&mut self) -> u16 {
        self.reader.get_u16()
    }

    pub fn u32(&mut self) -> u32 {
        self.reader.get_u32()
    }

    pub fn u64(&mut self) -> u64 {
        self.reader.get_u64()
    }

    pub fn i8(&mut self) -> i8 {
        self.reader.get_i8()
    }

    pub fn i16(&mut self) -> i16 {
        self.reader.get_i16()
    }

    pub fn i32(&mut self) -> i32 {
        self.reader.get_i32()
    }

    pub fn i64(&mut self) -> i64 {
        self.reader.get_i64()
    }

    pub fn bytes(&mut self) -> Bytes {
        self.reader.split_off(0)
    }

    pub fn string(&mut self) -> String {
        String::from_utf8_lossy(&self.bytes()).to_string()
    }

    pub fn skip(&mut self, n: usize) {
        self.reader.advance(n);
    }

    pub fn read_packet(&mut self, count: usize) -> BinaryPacket {
        BinaryPacket(self.reader.split_to(count))
    }

    pub fn read_with_length<T, const P: u8>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let length_counted = (P & PREFIX_WITH) > 0;
        let prefix_length = (P & 0b0111) as usize;
        let length = match prefix_length {
            0 => 0,
            1 => self.u8() as usize,
            2 => self.u16() as usize,
            4 => self.u32() as usize,
            _ => panic!("Invalid Prefix is given"),
        } - if length_counted { prefix_length } else { 0 };

        let buffer = self.reader.split_to(length);
        f(&mut PacketReader::new(buffer))
    }

    pub fn section<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.section_with_addition::<T, 4>(f)
    }

    pub fn section_with_addition<T, const N: isize>(
        &mut self,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let length = self.u32() as isize - N;
        let buffer = self.reader.split_to(length as usize);
        f(&mut PacketReader::new(buffer))
    }

    pub fn section_16<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.section_16_with_addition::<T, 2>(f)
    }

    pub fn section_16_with_addition<T, const N: isize>(
        &mut self,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let length = self.u16() as isize - N;
        let buffer = self.reader.split_to(length as usize);
        f(&mut PacketReader::new(buffer))
    }
}
