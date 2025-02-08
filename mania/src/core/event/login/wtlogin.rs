use crate::core::context::Context;
use crate::core::crypto::tea::tea_decrypt;
use crate::core::event::login::trans_emp::{build_wtlogin_packet, parse_wtlogin_packet};
use crate::core::event::prelude::*;
use crate::core::key_store::AccountInfo;
use crate::core::tlv::*;
use chrono::Utc;
use md5::{Digest, Md5};
use std::sync::Arc;

#[command("wtlogin.login")]
#[derive(Debug, Default, ServerEvent)]
pub struct WtLogin {
    pub code: i32,
    pub msg: Option<String>,
}

impl WtLogin {
    const BUILD_TLVS: [u16; 15] = [
        0x106, 0x144, 0x116, 0x142, 0x145, 0x018, 0x141, 0x177, 0x191, 0x100, 0x107, 0x318, 0x16A,
        0x166, 0x521,
    ];
}

impl ClientEvent for WtLogin {
    fn build(&self, context: &Context) -> Result<BinaryPacket, EventError> {
        let body = PacketBuilder::new()
            .u16(0x09)
            .packet(|p| serialize_tlv_set(context, Self::BUILD_TLVS.as_slice(), p))
            .build();
        let body = build_wtlogin_packet(context, 2064, &body);
        Ok(BinaryPacket(body.into()))
    }

    fn parse(packet: Bytes, ctx: &Context) -> Result<Box<dyn ServerEvent>, EventError> {
        let packet = parse_wtlogin_packet(packet, ctx)?;
        let mut reader = PacketReader::new(packet);
        reader.skip(2);
        let typ = reader.u8();
        let original_tlvs = TlvSet::deserialize(reader.bytes());
        if typ == 0 {
            let enc_tlvs_data = original_tlvs
                .get::<t119::T119>()
                .map_err(TlvError::MissingTlv)?
                .encrypted_tlv
                .to_owned();
            let dec_tlvs_data = tea_decrypt(&enc_tlvs_data, &ctx.session.stub.tgtgt_key.load());
            let tlvs = TlvSet::deserialize(Bytes::from(dec_tlvs_data));
            let tgt = tlvs
                .get::<t10a::T10A>()
                .map_err(TlvError::MissingTlv)?
                .tgt
                .to_owned();
            let d2 = tlvs
                .get::<t143::T143>()
                .map_err(TlvError::MissingTlv)?
                .d2
                .to_owned();
            let d2_key = tlvs
                .get::<t305::T305>()
                .map_err(TlvError::MissingTlv)?
                .d2key
                .to_owned();
            let trans_d2_key: &[u8; 16] = (&d2_key[..])
                .try_into()
                .expect("d2_key is not 16 bytes long");
            let tgtgt = Bytes::copy_from_slice(&Md5::digest(&d2_key));
            let temp_pwd = tlvs
                .get::<t106::T106>()
                .map_err(TlvError::MissingTlv)?
                .temp
                .to_owned();
            let uid = tlvs
                .get::<t543::T543>()
                .map_err(TlvError::MissingTlv)?
                .uid
                .to_owned();
            let self_info = tlvs.get::<t11a::T11A>().map_err(TlvError::MissingTlv)?;
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
                name: self_info.nick_name.to_owned(),
            }));
            Ok(Box::new(Self { code: 0, msg: None }))
        } else {
            let tlv146 = original_tlvs
                .get::<t146::T146>()
                .map_err(TlvError::MissingTlv)?;
            Ok(Box::new(Self {
                code: typ as i32,
                msg: Some(tlv146.message.to_owned()),
            }))
        }
    }
}
