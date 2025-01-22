use crate::crypto::consts::ECDH_256_PEER_KEY;
use md5::{Digest, Md5};
use p256::{ecdh::EphemeralSecret, EncodedPoint, PublicKey};

mod consts;
pub mod tea;

/// The original macro that @wybxc originally wrote (b81f75b7) was perfect.
/// but since that mania has dropped OpenSSL, it seems that this kind of abstraction is no longer needed. 
/// If there's ever a need in the future, we'll revisit it.
pub trait Ecdh {
    fn new() -> Self;
    fn public_key(&self) -> &[u8];
    fn shared_key(&self) -> &[u8];
    fn key_exchange<C>(
        c_pri_key: elliptic_curve::ecdh::EphemeralSecret<C>,
        s_pub_key: elliptic_curve::PublicKey<C>,
    ) -> [u8; 16]
    where
        C: elliptic_curve::CurveArithmetic,
    {
        let share = c_pri_key.diffie_hellman(&s_pub_key);
        let share_slice: [u8; 16] = share.raw_secret_bytes()[0..16].try_into().unwrap();
        let result = Md5::digest(share_slice);
        let mut shared_key = [0; 16];
        shared_key.copy_from_slice(&result);
        shared_key
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
    fn new() -> Self {
        let s_pub_key =
            PublicKey::from_sec1_bytes(&ECDH_256_PEER_KEY).expect("Failed to parse public key");
        let c_pri_key = EphemeralSecret::random(&mut rand::thread_rng());
        let c_pub_key = c_pri_key.public_key();
        let share_key = Self::key_exchange(c_pri_key, s_pub_key);
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
    use rand::thread_rng;
    use super::*;
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
        let ciphertext_from_client = tea::tea_encrypt(client_message, &client_shared);
        let decrypted_by_server = tea::tea_decrypt(&ciphertext_from_client, &server_shared);
        assert_eq!(client_message.to_vec(), decrypted_by_server);
        let server_message = b"https://music.163.com/song?id=1921741824";
        let ciphertext_from_server = tea::tea_encrypt(server_message, &server_shared);
        let decrypted_by_client = tea::tea_decrypt(&ciphertext_from_server, &client_shared);
        assert_eq!(server_message.to_vec(), decrypted_by_client);
        
        println!("Client message: {:?}", String::from_utf8_lossy(client_message));
        println!("Ciphertext from client: {:?}", ciphertext_from_client);
        println!("Decrypted by server: {:?}", String::from_utf8_lossy(&decrypted_by_server));
        println!("Server message: {:?}", String::from_utf8_lossy(server_message));
        println!("Ciphertext from server: {:?}", ciphertext_from_server);
        println!("Decrypted by client: {:?}", String::from_utf8_lossy(&decrypted_by_client));
    }
}
