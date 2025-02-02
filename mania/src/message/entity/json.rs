use super::prelude::*;

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
    fn pack_element(&self) -> Vec<Elem> {
        let mut template1 = zlib::compress(self.json.as_bytes());
        template1.push(0x01);
        vec![
            dda!(Elem {
                text: Some(dda!(Text {
                    str: Some(self.res_id.clone()),
                })),
            }),
            dda!(Elem {
                rich_msg: Some(dda!(RichMsg {
                    service_id: Some(1),
                    template1: Some(template1),
                }),),
            }),
        ]
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let rich_msg = elem.rich_msg.as_ref()?;
        match rich_msg.service_id? {
            1 => {
                let template1 = rich_msg.template1.as_ref()?;
                let mut data = zlib::decompress(template1)?;
                data.pop()?;
                let json = String::from_utf8(data).ok()?;
                let res_id = elem.text.as_ref()?.str.as_ref()?.clone();
                Some(JsonEntity { json, res_id })
            }
            _ => None,
        }
    }
}
