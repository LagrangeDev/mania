use super::prelude::*;

#[derive(Default)]
pub struct XmlEntity {
    pub xml: String,
    pub service_id: i32 = 35,
}

impl Debug for XmlEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[Xml]: {}", self.xml)
    }
}

impl Display for XmlEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[xml消息]")
    }
}

impl MessageEntity for XmlEntity {
    fn pack_element(&self) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let rich_msg = elem.rich_msg.as_ref()?;
        let (service_id, temp) = (rich_msg.service_id?, rich_msg.template1.as_ref()?);
        {
            let xml = zlib::decompress(&temp[1..])?;
            let xml = String::from_utf8(xml).ok()?;
            Some(XmlEntity { xml, service_id })
        }
    }
}
