use super::prelude::*;
use std::iter::once;

#[pack_content(false)]
#[derive(Default)]
pub struct JsonEntity {
    pub json: String,
    pub res_id: String,
}

impl Debug for JsonEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[Json]: {}", self.json)
    }
}

impl Display for JsonEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[Json消息]")
    }
}

impl MessageEntity for JsonEntity {
    fn pack_element(&self, _: &Context) -> Vec<Elem> {
        vec![
            dda!(Elem {
                text: Some(dda!(Text {
                    str: Some(self.res_id.clone()),
                })),
            }),
            dda!(Elem {
                rich_msg: Some(dda!(RichMsg {
                    service_id: Some(1),
                    template1: Some(
                        once(0x01)
                            .chain(zlib::compress(self.json.as_bytes()))
                            .collect()
                    ),
                }),),
            }),
        ]
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let rich_msg = elem.rich_msg.as_ref()?;
        match rich_msg.service_id? {
            1 => {
                let template1 = rich_msg.template1.as_ref()?;
                let data = zlib::decompress(&template1[1..])?;
                let json = String::from_utf8(data).ok()?;
                Some(dda!(JsonEntity { json: json }))
            }
            _ => None,
        }
    }
}
