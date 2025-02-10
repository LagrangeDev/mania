use super::prelude::*;

#[derive(Default)]
pub struct ForwardEntity {
    pub time: DateTime<Utc>,
    pub message_id: MessageId,
    pub sequence: u32,
    pub client_sequence: ClientSequence,
    pub uid: Option<String>,
    pub target_uin: u32,
    pub(crate) elems: Vec<Elem>,
    pub(crate) self_uid: Option<String>,
}

impl Debug for ForwardEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[Forward] Time: {} Sequence: {}",
            self.time, self.sequence
        )
    }
}

impl Display for ForwardEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[回复消息]")
    }
}

impl MessageEntity for ForwardEntity {
    fn pack_element(&self) -> Vec<Elem> {
        let pb_reserve = dda!(Preserve {
            message_id: self.message_id.0,
            sender_uid: self.uid.clone(),
        })
        .encode_to_vec();
        vec![dda!(Elem {
            src_msg: Some(dda!(SrcMsg {
                orig_seqs: vec![self.message_id.0 as u32],
                sender_uin: self.target_uin as u64,
                time: Some(self.time.timestamp() as i32),
                elems: self.elems.clone(),
                pb_reserve: Some(pb_reserve),
                to_uin: Some(0)
            }))
        })]
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        match elem.src_msg.as_ref() {
            Some(src) => {
                let reserve = Preserve::decode(&*src.pb_reserve.clone()?).ok()?;
                Some(dda!(Self {
                    time: DateTime::from_timestamp(src.time.unwrap_or(0) as i64, 0)?,
                    sequence: reserve.client_sequence.unwrap_or(src.orig_seqs[0]),
                    target_uin: src.sender_uin as u32,
                    message_id: MessageId(reserve.message_id),
                }))
            }
            None => None,
        }
    }
}
