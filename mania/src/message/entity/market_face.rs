use super::prelude::*;
use std::fmt::Debug;

#[derive(Default)]
pub struct MarketFaceEntity {
    pub emoji_id: String,
    pub emoji_package_id: u32,
    pub key: String,
    pub summary: String,
}

impl Debug for MarketFaceEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[MarketFace]: {} FaceId: {} TabId: {} Key: {}",
            self.summary, self.emoji_id, self.emoji_package_id, self.key
        )
    }
}

impl Display for MarketFaceEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.summary)
    }
}

impl MessageEntity for MarketFaceEntity {
    fn pack_element(&self) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        elem.market_face.as_ref().map(|market_face| Self {
            emoji_id: hex::encode(market_face.face_id.as_ref().unwrap()),
            emoji_package_id: market_face.tab_id.unwrap(),
            key: market_face.key.clone().unwrap(),
            summary: market_face.face_name.clone().unwrap(),
        })
    }
}
