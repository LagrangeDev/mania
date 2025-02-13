use arc_swap::{ArcSwap, ArcSwapOption};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeyStore {
    pub(crate) uin: ArcSwap<u32>,
    pub(crate) uid: ArcSwapOption<String>,
    pub(crate) password_md5: ArcSwap<Bytes>,
    pub(crate) session: WtLoginSession,
    pub(crate) info: ArcSwap<AccountInfo>,
}

impl KeyStore {
    pub fn load(file_path: &str) -> io::Result<KeyStore> {
        let file = fs::File::open(file_path)?;
        let reader = io::BufReader::new(file);
        let info: KeyStore = serde_json::from_reader(reader)?;
        Ok(info)
    }

    pub fn save(&self, file_path: &str) -> io::Result<()> {
        let file = fs::File::create(file_path)?;
        let writer = io::BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn is_session_expired(&self) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(**self.session.session_date.load());
        duration.num_seconds() > 15 * 86400
    }

    pub fn is_expired(&self) -> bool {
        self.is_session_expired()
            || self.session.d2.load().is_empty()
            || self.session.tgt.load().is_empty()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct AccountInfo {
    pub age: u8,
    pub gender: u8,
    pub name: ArcSwap<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct WtLoginSession {
    pub d2_key: ArcSwap<[u8; 16]>,
    pub d2: ArcSwap<Bytes>,
    pub tgt: ArcSwap<Bytes>,
    pub session_date: ArcSwap<DateTime<Utc>>,
    pub qr_sign: ArcSwapOption<Bytes>,
    pub qr_string: ArcSwapOption<String>,
    pub qr_url: ArcSwapOption<String>,
    pub exchange_key: ArcSwapOption<Bytes>,
    pub key_sign: ArcSwapOption<Bytes>,
    pub unusual_sign: ArcSwapOption<Bytes>,
    pub unusual_cookie: ArcSwapOption<String>,
    pub temp_password: ArcSwapOption<Bytes>,
    pub no_pic_sig: ArcSwapOption<Bytes>,
}

mod serde_rwlock {
    use serde::{Deserialize, Serialize, de, ser};
    use tokio::sync::RwLock;

    pub fn serialize<T, S>(value: &RwLock<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: ser::Serializer,
    {
        let read_guard = futures::executor::block_on(value.read());
        read_guard.serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<RwLock<T>, D::Error>
    where
        T: Deserialize<'de>,
        D: de::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(RwLock::new(value))
    }
}
