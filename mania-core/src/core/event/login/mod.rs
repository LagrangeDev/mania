pub mod wt_login;
pub mod wt_trans_emp;

use crate::core::context::Context;
use crate::core::crypto::ecdh::Ecdh;
use crate::core::packet::{PREFIX_U16, PREFIX_WITH, PacketBuilder, PacketError, PacketReader};
use crate::core::session::Session;
use bytes::Bytes;

// TODO: decouple
pub(crate) fn build_wtlogin_packet(ctx: &Context, cmd: u16, body: &[u8]) -> Vec<u8> {
    PacketBuilder::new()
        .u8(2) // packet start
        .write_with_length::<_, { PREFIX_U16 | PREFIX_WITH }, 1>(|packet| {
            packet
                .u16(8001) // ver
                .u16(cmd) // cmd: wtlogin.trans_emp: 2066, wtlogin.login: 2064
                .u16(ctx.session.next_sequence()) // unique wtLoginSequence for wtlogin packets only, should be stored in KeyStore
                .u32(**ctx.session.key_store.uin.load()) // uin, 0 for wt
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
                .u16(ctx.session.crypto.login_p256.public_key().len() as u16) // pubKey length
                .bytes(ctx.session.crypto.login_p256.public_key()) // pubKey
                .bytes(ctx.session.crypto.login_p256.tea_encrypt(body).as_slice())
                .u8(3) // packet end
        })
        .build()
}

// TODO: decouple
pub fn parse_wtlogin_packet(packet: Bytes, sess: &Session) -> Result<Bytes, PacketError> {
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

    let decrypted = sess.crypto.login_p256.tea_decrypt(&encrypted);

    Ok(Bytes::from(decrypted))
}
