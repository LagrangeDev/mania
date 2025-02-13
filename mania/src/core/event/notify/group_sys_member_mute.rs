use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysMemberMuteEvent {
    pub group_uin: u32,
    pub operator_uid: Option<String>,
    pub target_uid: String,
    pub duration: u32,
}
