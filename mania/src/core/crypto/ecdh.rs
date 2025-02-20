use crate::core::crypto::tea;
use digest::Digest;
use md5::Md5;
use p256::ecdh::EphemeralSecret;
use p256::{EncodedPoint, PublicKey};

pub const ECDH_256_PEER_LOGIN_KEY: [u8; 65] = [
    0x04, 0xEB, 0xCA, 0x94, 0xD7, 0x33, 0xE3, 0x99, 0xB2, 0xDB, 0x96, 0xEA, 0xCD, 0xD3, 0xF6, 0x9A,
    0x8B, 0xB0, 0xF7, 0x42, 0x24, 0xE2, 0xB4, 0x4E, 0x33, 0x57, 0x81, 0x22, 0x11, 0xD2, 0xE6, 0x2E,
    0xFB, 0xC9, 0x1B, 0xB5, 0x53, 0x09, 0x8E, 0x25, 0xE3, 0x3A, 0x79, 0x9A, 0xDC, 0x7F, 0x76, 0xFE,
    0xB2, 0x08, 0xDA, 0x7C, 0x65, 0x22, 0xCD, 0xB0, 0x71, 0x9A, 0x30, 0x51, 0x80, 0xCC, 0x54, 0xA8,
    0x2E,
];

pub const ECDH_256_PEER_EXCHANGE_KEY: [u8; 65] = [
    0x04, 0x9D, 0x14, 0x23, 0x33, 0x27, 0x35, 0x98, 0x0E, 0xDA, 0xBE, 0x7E, 0x9E, 0xA4, 0x51, 0xB3,
    0x39, 0x5B, 0x6F, 0x35, 0x25, 0x0D, 0xB8, 0xFC, 0x56, 0xF2, 0x58, 0x89, 0xF6, 0x28, 0xCB, 0xAE,
    0x3E, 0x8E, 0x73, 0x07, 0x79, 0x14, 0x07, 0x1E, 0xEE, 0xBC, 0x10, 0x8F, 0x4E, 0x01, 0x70, 0x05,
    0x77, 0x92, 0xBB, 0x17, 0xAA, 0x30, 0x3A, 0xF6, 0x52, 0x31, 0x3D, 0x17, 0xC1, 0xAC, 0x81, 0x5E,
    0x79,
];

pub const ECDH_192_PEER_KEY: [u8; 49] = [
    0x04, 0x92, 0x8D, 0x88, 0x50, 0x67, 0x30, 0x88, 0xB3, 0x43, 0x26, 0x4E, 0x0C, 0x6B, 0xAC, 0xB8,
    0x49, 0x6D, 0x69, 0x77, 0x99, 0xF3, 0x72, 0x11, 0xDE, 0xB2, 0x5B, 0xB7, 0x39, 0x06, 0xCB, 0x08,
    0x9F, 0xEA, 0x96, 0x39, 0xB4, 0xE0, 0x26, 0x04, 0x98, 0xB5, 0x1A, 0x99, 0x2D, 0x50, 0x81, 0x3D,
    0xA8,
];

/// The original macro that @wybxc originally wrote (b81f75b7) was perfect.
/// but since that mania has dropped OpenSSL, it seems that this kind of abstraction is no longer needed.
/// If there's ever a need in the future, we'll revisit it.
pub trait Ecdh {
    fn new(server_public_key: [u8; 65]) -> Self;
    fn public_key(&self) -> &[u8];
    fn shared_key(&self) -> &[u8];
    fn key_exchange<C>(
        c_pri_key: elliptic_curve::ecdh::EphemeralSecret<C>,
        s_pub_key: elliptic_curve::PublicKey<C>,
    ) -> Result<[u8; 16], String>
    where
        C: elliptic_curve::CurveArithmetic,
    {
        let share = c_pri_key.diffie_hellman(&s_pub_key);
        let share_slice: [u8; 16] = share.raw_secret_bytes()[0..16]
            .try_into()
            .map_err(|_| "Failed to convert shared secret to a fixed-size array".to_string())?;
        let result = Md5::digest(share_slice);
        let mut shared_key = [0; 16];
        shared_key.copy_from_slice(&result);
        Ok(shared_key)
    }
    fn tea_encrypt(&self, data: &[u8]) -> Vec<u8> {
        tea::tea_encrypt(data, self.shared_key())
    }
    fn tea_decrypt(&self, data: &[u8]) -> Vec<u8> {
        tea::tea_decrypt(data, self.shared_key())
    }
}

pub struct P256 {
    public: Vec<u8>,
    shared: [u8; 16],
}

impl Ecdh for P256 {
    fn new(server_public_key: [u8; 65]) -> Self {
        let s_pub_key =
            PublicKey::from_sec1_bytes(&server_public_key).expect("Failed to parse public key");
        let c_pri_key = EphemeralSecret::random(&mut rand::thread_rng());
        let c_pub_key = c_pri_key.public_key();
        let share_key = Self::key_exchange(c_pri_key, s_pub_key)
            .expect("Failed to generate shared key from key exchange");
        Self {
            public: EncodedPoint::from(c_pub_key).as_bytes().to_vec(),
            shared: share_key,
        }
    }

    fn public_key(&self) -> &[u8] {
        &self.public
    }

    fn shared_key(&self) -> &[u8] {
        &self.shared
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::thread_rng;
    #[test]
    fn test_ecdh_p256() {
        let mut rng = thread_rng();
        let server_secret = EphemeralSecret::random(&mut rng);
        let server_public = server_secret.public_key();
        let client_secret = EphemeralSecret::random(&mut rng);
        let client_public = client_secret.public_key();
        let server_pubkey = PublicKey::from_sec1_bytes(&server_public.to_sec1_bytes())
            .expect("failed to parse server public key");
        let client_pubkey = PublicKey::from_sec1_bytes(&client_public.to_sec1_bytes())
            .expect("failed to parse client public key");
        let server_shared = P256::key_exchange(server_secret, client_pubkey);
        let client_shared = P256::key_exchange(client_secret, server_pubkey);
        assert_eq!(server_shared, client_shared);

        let client_message = b"https://music.163.com/song?id=1496089150";
        let ciphertext_from_client =
            tea::tea_encrypt(client_message, &client_shared.clone().unwrap());
        let decrypted_by_server =
            tea::tea_decrypt(&ciphertext_from_client, &server_shared.clone().unwrap());
        assert_eq!(client_message.to_vec(), decrypted_by_server);
        let server_message = b"https://music.163.com/song?id=1921741824";
        let ciphertext_from_server =
            tea::tea_encrypt(server_message, &server_shared.clone().unwrap());
        let decrypted_by_client =
            tea::tea_decrypt(&ciphertext_from_server, &client_shared.unwrap());
        assert_eq!(server_message.to_vec(), decrypted_by_client);

        println!(
            "Client message: {:?}",
            String::from_utf8_lossy(client_message)
        );
        println!("Ciphertext from client: {:?}", ciphertext_from_client);
        println!(
            "Decrypted by server: {:?}",
            String::from_utf8_lossy(&decrypted_by_server)
        );
        println!(
            "Server message: {:?}",
            String::from_utf8_lossy(server_message)
        );
        println!("Ciphertext from server: {:?}", ciphertext_from_server);
        println!(
            "Decrypted by client: {:?}",
            String::from_utf8_lossy(&decrypted_by_client)
        );
    }
}
