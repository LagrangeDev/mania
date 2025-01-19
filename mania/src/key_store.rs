use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, io};
use tokio::sync::RwLock;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeyStore {
    #[serde(with = "serde_rwlock")]
    pub uin: RwLock<u32>,
    #[serde(with = "serde_rwlock")]
    pub uid: RwLock<Option<String>>,
    #[serde(with = "serde_rwlock")]
    pub password_md5: RwLock<Bytes>,
    pub session: WtLoginSession,
    #[serde(with = "serde_rwlock")]
    pub info: RwLock<AccountInfo>,
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

    pub async fn is_session_expired(&self) -> bool {
        let session_date = self.session.session_date.read().await;
        let now = Utc::now();
        let duration = now.signed_duration_since(*session_date);
        duration.num_seconds() > 15 * 86400
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccountInfo {
    pub(crate) age: u8,
    pub(crate) gender: u8,
    pub(crate) name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WtLoginSession {
    #[serde(with = "serde_rwlock")]
    pub d2_key: RwLock<[u8; 16]>,
    #[serde(with = "serde_rwlock")]
    pub d2: RwLock<Bytes>,
    #[serde(with = "serde_rwlock")]
    pub tgt: RwLock<Bytes>,
    #[serde(with = "serde_rwlock")]
    pub session_date: RwLock<DateTime<Utc>>,
    #[serde(with = "serde_rwlock")]
    pub qr_sign: RwLock<Option<Bytes>>,
    #[serde(with = "serde_rwlock")]
    pub qr_string: RwLock<Option<String>>,
    #[serde(with = "serde_rwlock")]
    pub qr_url: RwLock<Option<String>>,
    #[serde(with = "serde_rwlock")]
    pub exchange_key: RwLock<Option<Bytes>>,
    #[serde(with = "serde_rwlock")]
    pub key_sign: RwLock<Option<Bytes>>,
    #[serde(with = "serde_rwlock")]
    pub unusual_sign: RwLock<Option<Bytes>>,
    #[serde(with = "serde_rwlock")]
    pub unusual_cookie: RwLock<Option<String>>,
    #[serde(with = "serde_rwlock")]
    pub temp_password: RwLock<Option<Bytes>>,
    #[serde(with = "serde_rwlock")]
    pub no_pic_sig: RwLock<Option<Bytes>>,
}

mod serde_rwlock {
    use serde::{de, ser, Deserialize, Serialize};
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
