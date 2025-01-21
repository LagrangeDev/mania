use crate::context::Context;
use crate::crypto::tea::tea_decrypt;
use crate::event::trans_emp::{build_wtlogin_packet, parse_wtlogin_packet};
use crate::event::{ClientEvent, ParseEventError, ServerEvent};
use crate::key_store::AccountInfo;
use crate::packet::{BinaryPacket, PacketBuilder, PacketReader, PacketType};
use crate::tlv::*;
use bytes::Bytes;
use chrono::Utc;
use openssl::md::Md;
use openssl::md_ctx::MdCtx;
use std::fmt::Debug;
use std::sync::Arc;

pub struct WtLogin {}

impl WtLogin {
    const BUILD_TLVS: [u16; 15] = [
        0x106, 0x144, 0x116, 0x142, 0x145, 0x018, 0x141, 0x177, 0x191, 0x100, 0x107, 0x318, 0x16A,
        0x166, 0x521,
    ];
}

impl ClientEvent for WtLogin {
    fn command(&self) -> &'static str {
        "wtlogin.login"
    }

    fn packet_type(&self) -> PacketType {
        PacketType::T12
    }

    async fn build_packets(&self, context: &Context) -> Vec<BinaryPacket> {
        let body = PacketBuilder::new()
            .u16(0x09)
            .packet(|p| serialize_tlv_set(context, Self::BUILD_TLVS.as_slice(), p))
            .build();
        let body = build_wtlogin_packet(context, 2064, &body).await;
        vec![BinaryPacket(body.into())]
    }
}
#[derive(Debug, Clone)]
pub struct WtLoginRes {
    pub code: i32,
    pub msg: Option<String>,
}

impl ServerEvent for WtLoginRes {
    fn ret_code(&self) -> i32 {
        self.code
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn parse(packet: Bytes, ctx: &Context) -> Result<Vec<Box<dyn ServerEvent>>, ParseEventError> {
    let packet = parse_wtlogin_packet(packet, &ctx)?;
    let mut reader = PacketReader::new(packet);
    reader.skip(2);
    let typ = reader.u8();
    let original_tlvs = TlvSet::deserialize(reader.bytes());
    if typ == 0 {
        let enc_tlvs_data = original_tlvs
            .get::<t119::T119>()
            .map_err(ParseEventError::MissingTlv)?
            .encrypted_tlv
            .clone();
        let dec_tlvs_data = tea_decrypt(&enc_tlvs_data, &ctx.session.stub.tgtgt_key.load());
        let tlvs = TlvSet::deserialize(Bytes::from(dec_tlvs_data));
        let tgt = tlvs
            .get::<t10a::T10A>()
            .map_err(ParseEventError::MissingTlv)?
            .tgt
            .clone();
        let d2 = tlvs
            .get::<t143::T143>()
            .map_err(ParseEventError::MissingTlv)?
            .d2
            .clone();
        let d2_key = tlvs
            .get::<t305::T305>()
            .map_err(ParseEventError::MissingTlv)?
            .d2key
            .clone();
        let trans_d2_key: &[u8; 16] = (&d2_key[..])
            .try_into()
            .expect("d2_key is not 16 bytes long");
        let tgtgt = {
            let mut ctx = MdCtx::new().unwrap();
            ctx.digest_init(Md::md5()).unwrap();
            ctx.digest_update(&d2_key).unwrap();
            let mut buf = [0; 16];
            ctx.digest_final(&mut buf).unwrap();
            Bytes::copy_from_slice(&buf)
        };
        let temp_pwd = tlvs
            .get::<t106::T106>()
            .map_err(ParseEventError::MissingTlv)?
            .temp
            .clone();
        let uid = tlvs
            .get::<t543::T543>()
            .map_err(ParseEventError::MissingTlv)?
            .uid
            .clone();
        let self_info = tlvs
            .get::<t11a::T11A>()
            .map_err(ParseEventError::MissingTlv)?;
        ctx.session.stub.tgtgt_key.store(Arc::from(tgtgt));
        ctx.key_store.session.tgt.store(Arc::from(tgt));
        ctx.key_store.session.d2.store(Arc::from(d2));
        ctx.key_store
            .session
            .d2_key
            .store(Arc::from(trans_d2_key.to_owned()));
        ctx.key_store.uid.store(Some(Arc::from(uid)));
        ctx.key_store
            .session
            .temp_password
            .store(Some(Arc::from(temp_pwd)));
        ctx.key_store
            .session
            .session_date
            .store(Arc::from(Utc::now()));
        ctx.key_store.info.store(Arc::from(AccountInfo {
            age: self_info.age,
            gender: self_info.gender,
            name: self_info.nick_name.clone(),
        }));
        Ok(vec![Box::new(WtLoginRes { code: 0, msg: None })])
    } else {
        let tlv146 = original_tlvs.get::<t146::T146>().ok();
        Ok(vec![Box::new(WtLoginRes {
            code: typ as i32,
            msg: tlv146.map(|t| t.message.clone()),
        })])
    }
}
