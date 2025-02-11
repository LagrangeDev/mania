use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysRecallEvent {
    pub group_uin: u32,
    pub author_uid: String,
    pub operator_uid: Option<String>,
    pub sequence: u32,
    pub time: u32,
    pub random: u32,
    pub tip: String,
}
