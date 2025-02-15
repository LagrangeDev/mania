use super::prelude::*;

#[pack_content(false)]
#[derive(Default)]
pub struct TextEntity {
    pub text: String,
}

impl Debug for TextEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[Text]: {}", self.text)
    }
}

impl Display for TextEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.text)
    }
}

impl MessageEntity for TextEntity {
    fn pack_element(&self, _: &str) -> Vec<Elem> {
        vec![dda!(Elem {
            text: Some(dda!(Text {
                str: Some(self.text.clone()),
            })),
        })]
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let text = elem.text.as_ref()?;
        match (text.str.as_ref(), text.attr6_buf.as_ref()) {
            (Some(s), None) => Some(Self { text: s.clone() }),
            (Some(s), Some(buf)) if buf.is_empty() => Some(Self { text: s.clone() }),
            _ => None,
        }
    }
}
