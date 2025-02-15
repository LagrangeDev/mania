use super::prelude::*;

#[pack_content(false)]
#[derive(Default, Debug)]
pub struct ExtraInfoEntity {
    pub group_member_special_title: Option<String>,
}

impl Display for ExtraInfoEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "") // dummy
    }
}

impl MessageEntity for ExtraInfoEntity {
    fn pack_element(&self, _: &str) -> Vec<Elem> {
        unreachable!("Cannot pack CommonElemExtraEntity")
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        elem.extra_info.as_ref().map(|extra_info| Self {
            group_member_special_title: extra_info
                .sender_title
                .to_owned()
                .map(|title| String::from_utf8_lossy(&title).to_string()),
        })
    }
}
