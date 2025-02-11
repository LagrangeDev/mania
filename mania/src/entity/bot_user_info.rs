use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BotUserInfo {
    pub uin: u32,
    pub avatar: String,
    pub nickname: String,
    pub birthday: DateTime<Utc>,
    pub city: String,
    pub country: String,
    pub school: String,
    pub age: u32,
    pub register_time: DateTime<Utc>,
    pub gender: GenderInfo,
    pub qid: Option<String>,
    pub level: u32,
    pub sign: String,
    pub status: BotStatus,
    pub business: Vec<BusinessCustom>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessCustom {
    pub bus_type: u32,
    pub level: u32,
    pub icon: Option<String>,
    pub is_year: u32,
    pub is_pro: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessCustomList {
    pub business_lists: Vec<BusinessCustom>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum GenderInfo {
    #[default]
    Unset = 0,
    Male = 1,
    Female = 2,
    Unknown = 255,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BotStatus {
    pub status_id: u32,
    pub face_id: Option<u32>,
    pub msg: Option<String>,
}
