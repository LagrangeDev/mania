use super::prelude::*;
use std::fmt::Debug;

#[pack_content(false)]
#[derive(Default)]
pub struct MentionEntity {
    pub uin: u32,
    pub uid: String,
    pub name: Option<String>,
}

impl Debug for MentionEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[Mention]: {}({})",
            self.name.clone().unwrap_or_default(),
            self.uin
        )
    }
}

impl Display for MentionEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "@{}", self.name.clone().unwrap_or_default())
    }
}

impl MessageEntity for MentionEntity {
    fn pack_element(&self, _: &Context) -> Vec<Elem> {
        let pb_reserve = MentionExtra {
            r#type: Some(if self.uin == 0 { 1 } else { 2 }),
            uin: Some(self.uin),
            field5: Some(0),
            uid: Some(self.uid.clone()),
        }
        .encode_to_vec();
        vec![dda!(Elem {
            text: Some(dda!(Text {
                str: self.name.clone(),
                pb_reserve: Some(pb_reserve),
            }))
        })]
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let text = elem.text.as_ref()?;
        match (text.str.as_ref(), text.attr6_buf.as_ref()) {
            (Some(s), Some(buf)) if buf.len() >= 11 => {
                let uin = u32::from_be_bytes(
                    text.attr6_buf.as_ref()?[7..11]
                        .try_into()
                        .expect("slice length is 4"),
                );
                Some(dda!(Self {
                    uin,
                    name: Some(s.clone())
                }))
            }
            _ => None,
        }
    }
}
