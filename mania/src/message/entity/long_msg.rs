use super::prelude::*;
use crate::message::chain::MessageChain;

#[derive(Default)]
pub struct LongMsgEntity {
    pub res_id: String,
    pub chain: MessageChain,
}

impl Debug for LongMsgEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[LongMsg]: res_id: {} chains: {:?}",
            self.res_id, self.chain
        )
    }
}

impl Display for LongMsgEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[长消息]")
    }
}

impl MessageEntity for LongMsgEntity {
    fn pack_element(&self) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        let general = elem.general_flags.as_ref()?;
        match (general.long_text_res_id.as_ref(), general.long_text_flag) {
            (Some(res_id), 1) => Some(dda!(Self {
                res_id: res_id.clone(),
            })),
            _ => None,
        }
    }
}
