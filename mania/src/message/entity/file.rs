use super::prelude::*;
use crate::core::packet::{PacketReader, PREFIX_LENGTH_ONLY, PREFIX_U16};
use std::fmt::Debug;

#[derive(Default)]
pub struct FileGroupUnique {
    pub file_id: Option<String>,
}

#[derive(Default)]
pub struct FileC2CUnique {
    pub file_uuid: Option<String>,
    pub file_hash: Option<String>,
}

pub enum FileUnique {
    Group(FileGroupUnique),
    C2C(FileC2CUnique),
}

#[derive(Default)]
pub struct FileEntity {
    pub file_size: u64,
    pub file_name: String,
    pub file_md5: Bytes,
    pub file_url: Option<String>,
    pub(crate) file_sha1: Bytes,
    pub extra: Option<FileUnique>,
    // TODO: stream
}

impl Debug for FileEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[File]: {} ({}): {}",
            self.file_name,
            self.file_size,
            self.file_url
                .as_ref()
                .unwrap_or(&"failed to receive file url".to_string())
        )
    }
}

impl Display for FileEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[文件] {}", self.file_name)
    }
}

impl MessageEntity for FileEntity {
    fn pack_element(&self) -> Vec<Elem> {
        todo!()
    }

    fn pack_content(&self) -> Option<Bytes> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        match elem.trans_elem.as_ref()?.elem_type {
            24 => {
                let mut payload =
                    PacketReader::new(Bytes::from(elem.trans_elem.as_ref()?.elem_value.clone()));
                payload.skip(1);
                let data = payload
                    .read_with_length::<_, { PREFIX_U16 | PREFIX_LENGTH_ONLY }>(|p| p.bytes());
                let extra = GroupFileExtra::decode(data).ok()?.inner?.info?;
                Some(dda!(Self {
                    file_size: extra.file_size,
                    file_md5: Bytes::from(hex::decode(&extra.file_md5).ok()?),
                    file_name: extra.file_name,
                    extra: Some(FileUnique::Group(FileGroupUnique {
                        file_id: Some(extra.file_id.to_owned()),
                    })),
                }))
            }
            _ => None,
        }
    }
}
