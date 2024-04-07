use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeyStore {
    pub uin: u32,
    pub uid: Option<String>,
    pub password_md5: String,
    pub session: WtLoginSession,
    pub info: Option<AccountInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    age: u8,
    gender: u8,
    name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WtLoginSession {
    pub d2_key: [u8; 16],
    pub d2: Vec<u8>,
    pub tgt: Vec<u8>,
    pub session_date: DateTime<Utc>,
    pub temp_password: Option<Vec<u8>>,
}
