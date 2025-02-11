use crate::core::event::prelude::*;
use crate::core::protos::service::oidb::{
    Avatar, Business, CustomStatus, OidbSvcTrpcTcp0xFe12, OidbSvcTrpcTcp0xFe12key,
    OidbSvcTrpcTcp0xFe12response, OidbSvcTrpcTcp0xFe12responseBody,
    OidbSvcTrpcTcp0xFe12responseProperty, OidbSvcTrpcTcp0xFe12uin, business_list,
};
use crate::entity::bot_user_info::{BotStatus, BotUserInfo, BusinessCustom, GenderInfo};
use chrono::{DateTime, TimeZone, Utc};
use std::time::{Duration, UNIX_EPOCH};

#[oidb_command(0xfe1, 2)]
#[derive(Debug, ServerEvent, Default)]
pub struct FetchUserInfoEvent {
    pub user_info: BotUserInfo,
    pub uin: u32,
    pub uid: Option<String>,
}

static KEYS: &[u32] = &[
    20002, 27394, 20009, 20031, 101, 103, 102, 20022, 20023, 20024, 24002, 27037, 27049, 20011,
    20016, 20021, 20003, 20004, 20005, 20006, 20020, 20026, 24007, 104, 105, 42432, 42362, 41756,
    41757, 42257, 27372, 42315, 107, 45160, 45161, 27406, 62026, 20037,
];

impl ClientEvent for FetchUserInfoEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let keys: Vec<OidbSvcTrpcTcp0xFe12key> = KEYS
            .iter()
            .map(|&k| OidbSvcTrpcTcp0xFe12key { key: k })
            .collect();
        if let Some(uid) = &self.uid {
            let request = OidbSvcTrpcTcp0xFe12 {
                uid: Some(uid.to_owned()),
                field2: 0,
                keys,
            };
            Ok(OidbPacket::new(0xfe1, 2, request.encode_to_vec(), false, false).to_binary())
        } else {
            let request = OidbSvcTrpcTcp0xFe12uin {
                uin: self.uin,
                field2: 0,
                keys,
            };
            Ok(OidbPacket::new(0xfe1, 2, request.encode_to_vec(), false, true).to_binary())
        }
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let response = OidbPacket::parse_into::<OidbSvcTrpcTcp0xFe12response>(packet)?;
        let body: OidbSvcTrpcTcp0xFe12responseBody = response
            .body
            .ok_or_else(|| EventError::OtherError("Missing response body".into()))?;
        let properties: OidbSvcTrpcTcp0xFe12responseProperty = body
            .properties
            .ok_or_else(|| EventError::OtherError("Missing properties".into()))?;

        let bytes_props: HashMap<u32, Vec<u8>> = properties
            .bytes_properties
            .into_iter()
            .map(|prop| (prop.code, prop.value.to_vec()))
            .collect();
        let number_props: HashMap<u32, u32> = properties
            .number_properties
            .into_iter()
            .map(|prop| (prop.number1, prop.number2))
            .collect();

        let get_birthday = |birthday: &str| -> DateTime<Utc> {
            let bytes = Bytes::from(birthday.to_owned());
            let mut reader = PacketReader::new(bytes);
            let year = reader.u16();
            let month = reader.u8();
            let day = reader.u8();
            Utc.with_ymd_and_hms(year as i32, month as u32, day as u32, 0, 0, 0)
                .single()
                .unwrap_or_else(|| Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).single().unwrap())
        };

        let birthday = bytes_props
            .get(&20031)
            .and_then(|v| String::from_utf8(v.to_owned()).ok())
            .map(|s| get_birthday(&s))
            .unwrap_or_default();

        let reg_secs = *number_props.get(&20026).unwrap_or(&0) as u64;
        let register_time = DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(reg_secs));

        let qid = bytes_props
            .get(&27394)
            .and_then(|v| String::from_utf8(v.to_owned()).ok());

        let mut status_id = number_props.get(&27372).copied().unwrap_or(0);
        if status_id > 268435455 {
            status_id -= 268435456;
        }

        let customs = bytes_props.get(&27406).and_then(|bytes| {
            if !bytes.is_empty() {
                CustomStatus::decode(&bytes[..]).ok()
            } else {
                None
            }
        });

        let avatars = bytes_props
            .get(&101)
            .and_then(|bytes| Avatar::decode(&bytes[..]).ok())
            .unwrap_or_default();

        let business_a = bytes_props
            .get(&107)
            .and_then(|bytes| Business::decode(&bytes[..]).ok())
            .and_then(|business| {
                business.body.map(|body| {
                    body.lists
                        .into_iter()
                        .map(|b| BusinessCustom {
                            bus_type: b.r#type,
                            level: b.level,
                            icon: match &b.icon {
                                Some(business_list::Icon::Icon1(s)) if !s.is_empty() => {
                                    Some(s.to_owned())
                                }
                                Some(business_list::Icon::Icon2(s)) if !s.is_empty() => {
                                    Some(s.to_owned())
                                }
                                _ => None,
                            },
                            is_pro: b.is_pro as u32,
                            is_year: b.is_year as u32,
                        })
                        .collect()
                })
            })
            .unwrap_or_default();

        let nickname = bytes_props
            .get(&20002)
            .cloned()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();
        let city = bytes_props
            .get(&20020)
            .cloned()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();
        let country = bytes_props
            .get(&20003)
            .cloned()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();
        let school = bytes_props
            .get(&20021)
            .cloned()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();
        let sign = bytes_props
            .get(&102)
            .cloned()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();

        let gender = match *number_props.get(&20009).unwrap_or(&0) {
            1 => GenderInfo::Male,
            2 => GenderInfo::Female,
            _ => GenderInfo::Unset,
        };

        let user_info = BotUserInfo {
            uin: body.uin,
            nickname,
            avatar: format!("{}640", avatars.url.unwrap_or_default()),
            birthday,
            city,
            country,
            school,
            age: *number_props.get(&20037).unwrap_or(&0),
            register_time,
            gender,
            qid,
            level: *number_props.get(&105).unwrap_or(&0),
            sign,
            status: BotStatus {
                status_id,
                face_id: customs.as_ref().map(|c| c.face_id),
                msg: customs.and_then(|c| c.msg),
            },
            business: business_a,
        };

        Ok(ClientResult::single(Box::new(FetchUserInfoEvent {
            user_info,
            uin: body.uin,
            uid: None,
        })))
    }
}
