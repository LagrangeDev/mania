use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysMuteEvent {
    pub group_uin: u32,
    pub operator_uid: Option<String>,
    pub is_muted: bool,
}
