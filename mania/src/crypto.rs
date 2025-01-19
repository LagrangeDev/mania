use std::ops::Deref;

// FIXME: do not use openssl, use p256 instead
use openssl::bn::BigNumContext;
use openssl::derive::Deriver;
use openssl::ec::{EcGroup, EcKey, EcPoint, PointConversionForm};
use openssl::error::ErrorStack;
use openssl::md::Md;
use openssl::md_ctx::MdCtx;
use openssl::nid::Nid;
use openssl::pkey::{PKey, PKeyRef, Private, Public};

mod consts;
pub mod tea;

pub struct Ecdh {
    public: Vec<u8>,
    shared: [u8; 16],
}

macro_rules! ecdh_impl {
    ($curve:ident, $nid:expr, $pkey:expr) => {
        pub struct $curve(pub Ecdh);

        impl $curve {
            pub fn new() -> Result<Self, ErrorStack> {
                Ok(Self(Ecdh::new($nid, &$pkey)?))
            }
        }

        impl Deref for $curve {
            type Target = Ecdh;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

impl Ecdh {
    pub fn new(curve: Nid, pkey: &[u8]) -> Result<Self, ErrorStack> {
        let group = EcGroup::from_curve_name(curve)?;
        let key = EcKey::generate(&group)?;
        let mut context = BigNumContext::new()?;

        let public =
            key.public_key()
                .to_bytes(&group, PointConversionForm::COMPRESSED, &mut context)?;

        let shared = {
            let pkey = EcPoint::from_bytes(&group, pkey, &mut context)?;
            let pkey = EcKey::from_public_key(&group, &pkey)?;
            let pkey = PKey::from_ec_key(pkey)?;
            Ecdh::key_exchange(key.clone(), pkey.as_ref())?
        };

        Ok(Ecdh { public, shared })
    }

    /// Compressed public key
    pub fn public_key(&self) -> &[u8] {
        &self.public
    }

    /// ECDH key exchange, then MD5 and truncate to 16 bytes
    fn key_exchange(
        key: EcKey<Private>,
        peer_key: &PKeyRef<Public>,
    ) -> Result<[u8; 16], ErrorStack> {
        let key = PKey::from_ec_key(key)?;

        let mut deriver = Deriver::new(&key)?;
        deriver.set_peer(peer_key)?;
        let shared = deriver.derive_to_vec()?;

        let md5 = {
            let mut ctx = MdCtx::new()?;
            ctx.digest_init(Md::md5())?;
            ctx.digest_update(&shared)?;
            let mut buf = [0; 16];
            ctx.digest_final(&mut buf)?;
            buf
        };

        Ok(md5)
    }

    pub fn tea_encrypt(&self, data: &[u8]) -> Vec<u8> {
        tea::tea_encrypt(data, &self.shared)
    }

    pub fn tea_decrypt(&self, data: &[u8]) -> Vec<u8> {
        tea::tea_decrypt(data, &self.shared)
    }
}

ecdh_impl!(Secp192k1, Nid::SECP192K1, consts::ECDH_192_PEER_KEY);
ecdh_impl!(Prime256v1, Nid::X9_62_PRIME256V1, consts::ECDH_256_PEER_KEY);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ecdh() {
        let ecdh = Secp192k1::new().unwrap();
        let public_key = ecdh.public_key();
        assert_eq!(public_key.len(), 25);
        assert!(public_key[0] == 0x02 || public_key[0] == 0x03);
    }
}
