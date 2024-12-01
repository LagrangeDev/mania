use std::vec;

use bytes::Bytes;
use chrono::Utc;

use crate::context::Context;
use crate::event::{ClientEvent, ParseEventError, ServerEvent};
use crate::packet::{
    BinaryPacket, PacketBuilder, PacketReader, PacketType, PREFIX_U16, PREFIX_WITH,
};
use crate::tlv::t017::T017;
use crate::tlv::t018::T018;
use crate::tlv::t019::T019;
use crate::tlv::t01c::T01c;
use crate::tlv::t01e::T01e;
use crate::tlv::t0d1::T0d1Resp;
use crate::tlv::{serialize_tlv_set, TlvSet};

pub enum TransEmp {
    FetchQrCode,
    QueryResult,
}

impl TransEmp {
    const TLVS: [u16; 7] = [0x016, 0x01B, 0x01D, 0x033, 0x035, 0x066, 0x0D1];
    const TLVS_PASSWORD: [u16; 8] = [0x011, 0x016, 0x01B, 0x01D, 0x033, 0x035, 0x066, 0x0D1];
}

impl ClientEvent for TransEmp {
    fn command(&self) -> &'static str {
        "wtlogin.trans_emp"
    }

    fn packet_type(&self) -> PacketType {
        PacketType::T12
    }

    fn build_packets(&self, ctx: &Context) -> Vec<BinaryPacket> {
        let body = match self {
            TransEmp::FetchQrCode => build_trans_emp_body(ctx, 0x31, |packet| {
                // TransEmp31.ConstructTransEmp()
                let packet = packet.u16(0).u64(0).u8(0);

                let tags = if ctx.session.unusual_sign.is_none() {
                    Self::TLVS.as_slice()
                } else {
                    Self::TLVS_PASSWORD.as_slice()
                };

                packet.packet(|p| serialize_tlv_set(ctx, tags, p)).u8(0x03)
            }),
            TransEmp::QueryResult => build_trans_emp_body(ctx, 0x12, |packet| {
                // TransEmp12.ConstructTransEmp()
                let qrsign = ctx.session.qr_sign.load();
                let qrsign = qrsign.as_ref().expect("qr sign not initialized");

                packet
                    .u16(qrsign.sign.len() as u16) // sign length
                    .bytes(&qrsign.sign)
                    .u64(0) // const
                    .u32(0) // const
                    .u8(0) // const
                    .u8(0x03) // end of packet
            }),
        };
        let packet = build_wtlogin_packet(ctx, 2066, &body);
        vec![BinaryPacket(packet.into())]
    }
}

fn build_trans_emp_body(
    ctx: &Context,
    qr_cmd: u16,
    build_inner: impl FnOnce(PacketBuilder) -> PacketBuilder,
) -> Vec<u8> {
    PacketBuilder::new()
        .u8(0) // unknown
        // -13 is the length of zeros, which could be found at TransEmp31 and TransEmp12.ConstructTransEmp()
        .write_with_length::<_, { PREFIX_U16 | PREFIX_WITH }, -13>(|packet| {
            packet
                .u32(ctx.app_info.app_id as u32)
                .u32(0x72) // const
                .u16(0) // const
                .u8(0)
                .u32(Utc::now().timestamp() as u32) // length actually starts here
                .u8(2) // header for packet, counted into length of next barrier manually
                // addition 1 is the packet start counted in
                .write_with_length::<_, { PREFIX_U16 | PREFIX_WITH }, 1>(|packet| {
                    packet
                        .u16(qr_cmd)
                        .u64(0) // const
                        .u32(0) // const
                        .u64(0) // const
                        .u16(3) // const
                        .u16(0) // const
                        .u16(50) // unknown const
                        .u64(0)
                        .u32(0)
                        .u16(0)
                        .u32(ctx.app_info.app_id as u32)
                        .packet(build_inner)
                })
        })
        .build()
}

pub fn build_wtlogin_packet(ctx: &Context, cmd: u16, body: &[u8]) -> Vec<u8> {
    PacketBuilder::new()
        .u8(2) // packet start
        .write_with_length::<_, { PREFIX_U16 | PREFIX_WITH }, 1>(|packet| {
            packet
                .u16(8001) // ver
                .u16(cmd) // cmd: wtlogin.trans_emp: 2066, wtlogin.login: 2064
                .u16(ctx.session.next_sequence()) // unique wtLoginSequence for wtlogin packets only, should be stored in KeyStore
                .u32(ctx.key_store.uin) // uin, 0 for wt
                .u8(3) // extVer
                .u8(135) // cmdVer
                .u32(0) // actually unknown const 0
                .u8(19) // pubId
                .u16(0) // insId
                .u16(ctx.app_info.app_client_version) // cliType
                .u32(0) // retryTime
                .u8(1) // const
                .u8(1) // const
                .bytes(&ctx.session.stub.random_key) // randKey
                .u16(0x102) // unknown const
                .u16(25) // pubKey length
                .bytes(ctx.crypto.secp.public_key()) // pubKey
                .bytes(ctx.crypto.secp.tea_encrypt(body).as_slice())
                .u8(3) // packet end
        })
        .build()
}

#[derive(Debug)]
pub struct TransEmp31 {
    pub qr_code: Bytes,
    pub expiration: u32,
    pub url: String,
    pub qr_sig: String,
    pub signature: Bytes,
}

impl ServerEvent for TransEmp31 {
    fn ret_code(&self) -> i32 {
        0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum TransEmp12 {
    Confirmed {
        tgtgt_key: Bytes,
        temp_password: Bytes,
        no_pic_sig: Bytes,
    } = 0,
    CodeExpired = 17,
    WaitingForScan = 48,
    WaitingForConfirm = 53,
    Canceled = 54,
}

impl ServerEvent for TransEmp12 {
    fn ret_code(&self) -> i32 {
        match self {
            TransEmp12::Confirmed { .. } => 0,
            TransEmp12::CodeExpired => 17,
            TransEmp12::WaitingForScan => 48,
            TransEmp12::WaitingForConfirm => 53,
            TransEmp12::Canceled => 54,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn parse(packet: Bytes, ctx: &Context) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError> {
    // Lagrange.Core.Internal.Packets.Login.WtLogin.Entity.TransEmp.DeserializeBody
    let packet = parse_wtlogin_packet(packet, ctx)?;
    let mut reader = PacketReader::new(packet);

    let _packet_length = reader.u32();
    let _ = reader.u32(); // misc unknown data
    let command = reader.u16();
    reader.skip(40);
    let _app_id = reader.u32();

    let packet = reader.bytes();

    // Lagrange.Core.Internal.Service.Login.TransEmpService.Parse
    match command {
        0x31 => {
            // Lagrange.Core.Internal.Packets.Login.WtLogin.Entity.TransEmp31.Deserialize
            let mut reader = PacketReader::new(packet);
            let _ = reader.u8();
            let signature = reader.section_16_with_addition::<_, 0>(|p| p.bytes());
            let tlvs = TlvSet::deserialize(reader.bytes());

            let qr_code = tlvs
                .get::<T017>()
                .map_err(ParseEventError::MissingTlv)?
                .qr_code
                .clone();
            let expiration = tlvs
                .get::<T01c>()
                .map_err(ParseEventError::MissingTlv)?
                .expire_sec;
            let t0d1 = tlvs
                .get::<T0d1Resp>()
                .map_err(ParseEventError::MissingTlv)?;
            let url = t0d1.proto.url.clone();
            let qr_sig = t0d1.proto.qr_sig.clone();

            Ok(vec![Box::new(TransEmp31 {
                qr_code,
                expiration,
                url,
                qr_sig,
                signature,
            })])
        }
        0x12 => {
            // Lagrange.Core.Internal.Packets.Login.WtLogin.Entity.TransEmp12.Deserialize
            let mut reader = PacketReader::new(packet);
            let state = reader.u8();
            let result = match state {
                0 => {
                    reader.skip(12); // misc unknown data

                    let tlvs = TlvSet::deserialize(reader.bytes());
                    let tgtgt_key = tlvs
                        .get::<T01e>()
                        .map_err(ParseEventError::MissingTlv)?
                        .tgtgt_key
                        .clone();
                    let temp_password = tlvs
                        .get::<T018>()
                        .map_err(ParseEventError::MissingTlv)?
                        .temp_password
                        .clone();
                    let no_pic_sig = tlvs
                        .get::<T019>()
                        .map_err(ParseEventError::MissingTlv)?
                        .no_pic_sig
                        .clone();

                    TransEmp12::Confirmed {
                        tgtgt_key,
                        temp_password,
                        no_pic_sig,
                    }
                }
                17 => TransEmp12::CodeExpired,
                48 => TransEmp12::WaitingForScan,
                53 => TransEmp12::WaitingForConfirm,
                54 => TransEmp12::Canceled,
                _ => Err(ParseEventError::UnknownRetCode(state as i32))?,
            };
            Ok(vec![Box::new(result)])
        }
        _ => Err(ParseEventError::UnsupportedTransEmp(command)),
    }
}

pub fn parse_wtlogin_packet(packet: Bytes, ctx: &Context) -> Result<Bytes, ParseEventError> {
    // Lagrange.Core.Internal.Packets.Login.WtLogin.WtLoginBase.DeserializePacket
    let mut reader = PacketReader::new(packet);
    let header = reader.u8();
    if header != 2 {
        return Err(ParseEventError::InvalidPacketHeader);
    }

    let _length = reader.u16();
    let _ver = reader.u16();
    let _cmd = reader.u16();
    let _sequence = reader.u16();
    let _uin = reader.u32();
    let _flag = reader.u8();
    let _retry_time = reader.u16();

    let mut encrypted = reader.bytes();
    let tail = encrypted.split_off(encrypted.len() - 1)[0];
    if tail != 3 {
        return Err(ParseEventError::InvalidPacketEnd);
    }

    let decrypted = ctx.crypto.secp.tea_decrypt(&encrypted);

    Ok(Bytes::from(decrypted))
}
