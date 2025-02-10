use super::prelude::*;
use crate::core::protos::service::oidb::MsgInfo;

#[derive(Default)]
pub struct RecordEntity {
    pub audio_length: u32,
    pub audio_md5: Bytes = Bytes::new(),
    pub audio_name: String,
    pub audio_url: String,
    // TODO: stream
    pub(crate) audio_uuid: Option<String>,
    pub(crate) file_sha1: Option<String>,
    pub(crate) msg_info: Option<MsgInfo>,
    pub(crate) compat: Option<RichText>,
}

impl Debug for RecordEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[Record]: {}", self.audio_url)
    }
}

impl Display for RecordEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[语音]")
    }
}

impl MessageEntity for RecordEntity {
    fn pack_element(&self) -> Vec<Elem> {
        let common = self.msg_info.as_ref().map_or_else(
            || {
                MsgInfo {
                    ..Default::default()
                }
                .encode_to_vec()
            },
            |msg_info| msg_info.encode_to_vec(),
        );
        vec![dda!(Elem {
            common_elem: Some(CommonElem {
                service_type: 48,
                pb_elem: common,
                business_type: 22,
            }),
        })]
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let common_elem = elem.common_elem.as_ref()?;
        match (common_elem.business_type, common_elem.service_type) {
            (22 | 12, 48) => {
                let extra = MsgInfo::decode(&*common_elem.pb_elem).ok()?;
                let index = &extra.msg_info_body.first()?.index.as_ref()?;
                let (uuid, name, sha1) = (
                    &index.file_uuid,
                    &index.info.as_ref()?.file_name,
                    &index.info.as_ref()?.file_hash,
                );
                {
                    let md5 = Bytes::from(hex::decode(sha1).ok()?);
                    Some(dda!(Self {
                        audio_uuid: Some(uuid.to_owned()),
                        audio_name: name.to_owned(),
                        audio_md5: md5,
                        audio_length: index.info.as_ref()?.time,
                        file_sha1: Some(sha1.to_owned()),
                        msg_info: Some(extra.to_owned()),
                    }))
                }
            }
            _ => None,
        }
    }
}
