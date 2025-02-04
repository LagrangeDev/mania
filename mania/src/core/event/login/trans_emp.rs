use crate::core::crypto::Ecdh;
use crate::core::event::prelude::*;
use crate::core::tlv::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct NTLoginHttpRequest {
    pub appid: u64,
    #[serde(rename = "faceUpdateTime")]
    pub face_update_time: u64,
    pub qrsig: String,
}

#[derive(Debug, Deserialize)]
pub struct NTLoginHttpResponse {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
    #[serde(rename = "qrSig")]
    pub qr_sig: String,
    #[serde(rename = "uin")]
    pub uin: u32,
    #[serde(rename = "faceUrl")]
    pub face_url: String,
    #[serde(rename = "faceUpdateTime")]
    pub face_update_time: i64,
}

#[commend("wtlogin.trans_emp")]
#[derive(Debug, ServerEvent)]
pub struct TransEmp {
    pub status: TransEmpStatus,
    pub result: Option<TransEmpResult>,
}

#[repr(u16)]
#[derive(Debug)]
pub enum TransEmpStatus {
    QueryResult = 0x12,
    FetchQrCode = 0x31,
}

#[derive(Debug)]
pub enum TransEmpResult {
    Emp12(TransEmp12Res),
    Emp31(TransEmp31Res),
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum TransEmp12Res {
    Confirmed(TransEmp12ConfirmedData) = 0,
    CodeExpired = 17,
    WaitingForScan = 48,
    WaitingForConfirm = 53,
    Canceled = 54,
}

#[derive(Debug, Clone)]
pub struct TransEmp12ConfirmedData {
    pub tgtgt_key: Bytes,
    pub temp_password: Bytes,
    pub no_pic_sig: Bytes,
}

#[derive(Debug)]
pub struct TransEmp31Res {
    pub qr_code: Bytes,
    pub expiration: u32,
    pub url: String,
    pub qr_sig: String,
    pub signature: Bytes,
}

impl TransEmp {
    const TLVS: [u16; 7] = [0x016, 0x01B, 0x01D, 0x033, 0x035, 0x066, 0x0D1];
    const TLVS_PASSWORD: [u16; 8] = [0x011, 0x016, 0x01B, 0x01D, 0x033, 0x035, 0x066, 0x0D1];

    pub fn new_fetch_qr_code() -> Self {
        Self {
            status: TransEmpStatus::FetchQrCode,
            result: None,
        }
    }

    pub fn new_query_result() -> Self {
        Self {
            status: TransEmpStatus::QueryResult,
            result: None,
        }
    }
}

impl ClientEvent for TransEmp {
    fn build(&self, ctx: &Context) -> BinaryPacket {
        let body = match self.status {
            TransEmpStatus::QueryResult => {
                let qrsign = ctx.session.qr_sign.load();
                let qrsign = qrsign.as_ref().expect("qr sign not initialized");
                let data = PacketBuilder::new()
                    .u16(0)
                    .u32(ctx.app_info.app_id as u32)
                    .write_with_length::<_, { PREFIX_U16 | PREFIX_LENGTH_ONLY }, 0>(|packet| {
                        packet.bytes(&qrsign.sign)
                    })
                    .u64(0)
                    .u8(0)
                    .write_with_length::<_, { PREFIX_U16 | PREFIX_LENGTH_ONLY }, 0>(|packet| {
                        packet.bytes(&[])
                    })
                    .u16(0)
                    .build();
                build_trans_emp_body(ctx, 0x12, data)
            }
            TransEmpStatus::FetchQrCode => {
                let tlvs = if ctx.session.unusual_sign.is_none() {
                    Self::TLVS.as_slice()
                } else {
                    Self::TLVS_PASSWORD.as_slice()
                };
                let data = PacketBuilder::new()
                    .u16(0)
                    .u32(ctx.app_info.app_id as u32)
                    .u64(0)
                    .bytes(&[])
                    .u8(0)
                    .write_with_length::<_, { PREFIX_U16 | PREFIX_LENGTH_ONLY }, 0>(|packet| {
                        packet.bytes(&[])
                    })
                    .packet(|p| serialize_qrcode_tlv_set(ctx, tlvs, p))
                    .build();
                build_trans_emp_body(ctx, 0x31, data)
            }
        };
        let packet = build_wtlogin_packet(ctx, 2066, &body);
        BinaryPacket(packet.into())
    }

    fn parse(packet: Bytes, context: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        // Lagrange.Core.Internal.Packets.Login.WtLogin.Entity.TransEmp.DeserializeBody
        let packet = parse_wtlogin_packet(packet, context)?;
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
                let tlvs = TlvSet::deserialize_qrcode(reader.bytes());

                let qr_code = tlvs
                    .get::<t017q::T017q>()
                    .map_err(TlvError::MissingTlv)?
                    .qr_code
                    .clone();
                let expiration = tlvs
                    .get::<t01cq::T01cq>()
                    .map_err(TlvError::MissingTlv)?
                    .expire_sec;
                let t0d1 = tlvs
                    .get::<t0d1q::T0d1Resp>()
                    .map_err(TlvError::MissingTlv)?;
                let url = t0d1.proto.url.clone();
                let qr_sig = t0d1.proto.qr_sig.clone();

                Ok(Box::new(Self {
                    status: TransEmpStatus::FetchQrCode,
                    result: Some(TransEmpResult::Emp31(TransEmp31Res {
                        qr_code,
                        expiration,
                        url,
                        qr_sig,
                        signature,
                    })),
                }))
            }
            0x12 => {
                // Lagrange.Core.Internal.Packets.Login.WtLogin.Entity.TransEmp12.Deserialize
                let mut reader = PacketReader::new(packet);
                let state = reader.u8();
                let result = match state {
                    0 => {
                        reader.skip(12); // misc unknown data

                        let tlvs = TlvSet::deserialize_qrcode(reader.bytes());
                        let tgtgt_key = tlvs
                            .get::<t01eq::T01eq>()
                            .map_err(TlvError::MissingTlv)?
                            .tgtgt_key
                            .clone();
                        let temp_password = tlvs
                            .get::<t018q::T018q>()
                            .map_err(TlvError::MissingTlv)?
                            .temp_password
                            .clone();
                        let no_pic_sig = tlvs
                            .get::<t019q::T019q>()
                            .map_err(TlvError::MissingTlv)?
                            .no_pic_sig
                            .clone();

                        TransEmp12Res::Confirmed(TransEmp12ConfirmedData {
                            tgtgt_key,
                            temp_password,
                            no_pic_sig,
                        })
                    }
                    17 => TransEmp12Res::CodeExpired,
                    48 => TransEmp12Res::WaitingForScan,
                    53 => TransEmp12Res::WaitingForConfirm,
                    54 => TransEmp12Res::Canceled,
                    _ => Err(EventError::OtherError(format!(
                        "unknown trans_emp ret code: {}",
                        state
                    )))?,
                };
                Ok(Box::new(Self {
                    status: TransEmpStatus::QueryResult,
                    result: Some(TransEmpResult::Emp12(result)),
                }))
            }
            _ => Err(EventError::OtherError(format!(
                "unsupported trans_emp command: {:#x}",
                command
            )))?,
        }
    }
}

fn build_trans_emp_body(ctx: &Context, qr_cmd: u16, tlvs: Vec<u8>) -> Vec<u8> {
    let new_packet = PacketBuilder::new()
        .u8(2)
        .u16((43 + tlvs.len() + 1) as u16)
        .u16(qr_cmd)
        .bytes(&[0; 21])
        .u8(0x03)
        .u16(0x00)
        .u16(0x32)
        .u32(0)
        .u64(0)
        .bytes(&tlvs)
        .u8(3)
        .build();

    let request_body = PacketBuilder::new()
        .u32(Utc::now().timestamp() as u32)
        .bytes(&new_packet)
        .build();

    PacketBuilder::new()
        .u8(0x00)
        .u16(request_body.len() as u16)
        .u32(ctx.app_info.app_id as u32)
        .u32(0x72)
        .write_with_length::<_, { PREFIX_U16 | PREFIX_LENGTH_ONLY }, 0>(|packet| packet.bytes(&[]))
        .write_with_length::<_, { PREFIX_U8 | PREFIX_LENGTH_ONLY }, 0>(|packet| packet.bytes(&[]))
        .bytes(&request_body)
        .build()
}

// TODO: decouple
pub fn build_wtlogin_packet(ctx: &Context, cmd: u16, body: &[u8]) -> Vec<u8> {
    PacketBuilder::new()
        .u8(2) // packet start
        .write_with_length::<_, { PREFIX_U16 | PREFIX_WITH }, 1>(|packet| {
            packet
                .u16(8001) // ver
                .u16(cmd) // cmd: wtlogin.trans_emp: 2066, wtlogin.login: 2064
                .u16(ctx.session.next_sequence()) // unique wtLoginSequence for wtlogin packets only, should be stored in KeyStore
                .u32(**ctx.key_store.uin.load()) // uin, 0 for wt
                .u8(3) // extVer
                .u8(135) // cmdVer
                .u32(0) // actually unknown const 0
                .u8(19) // pubId
                .u16(0) // insId
                .u16(ctx.app_info.app_client_version) // cliType
                .u32(0) // retryTime
                // head
                .u8(2) // curve type (Secp192K1: 1, Prime256V1: 2)
                .u8(1) // rollback flag
                .bytes(&ctx.session.stub.random_key) // randKey
                .u16(0x0131) // android: 0x0131, windows: 0x0102
                .u16(0x0001)
                .u16(ctx.crypto.login_p256.public_key().len() as u16) // pubKey length
                .bytes(ctx.crypto.login_p256.public_key()) // pubKey
                .bytes(ctx.crypto.login_p256.tea_encrypt(body).as_slice())
                .u8(3) // packet end
        })
        .build()
}

// TODO: decouple
pub fn parse_wtlogin_packet(packet: Bytes, ctx: &Context) -> Result<Bytes, PacketError> {
    // Lagrange.Core.Internal.Packets.Login.WtLogin.WtLoginBase.DeserializePacket
    let mut reader = PacketReader::new(packet);
    let header = reader.u8();
    if header != 2 {
        return Err(PacketError::OtherError(
            "invalid packet header when parse_wtlogin_packet".to_string(),
        ));
    }

    reader.u16(); // length
    reader.u16(); // ver
    reader.u16(); // cmd
    reader.u16(); // seq
    reader.u32(); // uin
    reader.u8(); // flag
    reader.u16(); // retry time

    let mut encrypted = reader.bytes();
    let tail = encrypted.split_off(encrypted.len() - 1)[0];
    if tail != 3 {
        return Err(PacketError::OtherError(
            "invalid packet end when parse_wtlogin_packet".to_string(),
        ));
    }

    let decrypted = ctx.crypto.login_p256.tea_decrypt(&encrypted);

    Ok(Bytes::from(decrypted))
}
