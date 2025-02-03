use super::prelude::*;
use serde_json::Value;
use std::fmt::Debug;

#[derive(Default)]
pub struct LightAppEntity {
    pub app_name: String,
    pub payload: String,
}

impl Debug for LightAppEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[LightApp]: {}", self.app_name)
    }
}

impl Display for LightAppEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[轻应用消息]")
    }
}

impl MessageEntity for LightAppEntity {
    fn pack_element(&self) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let payload = elem.light_app_elem.as_ref().and_then(|light_app| {
            zlib::decompress(&light_app.data[1..])
                .and_then(|decompressed| String::from_utf8(decompressed).ok())
        })?;
        let parsed_payload: Value = serde_json::from_str(&payload).ok()?;
        let app = parsed_payload.get("app")?.as_str()?;
        Some(LightAppEntity {
            app_name: app.to_string(),
            payload,
        })
    }
}
