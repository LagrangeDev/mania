use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysRequestJoinEvent {
    pub target_uid: String,
    pub group_uin: u32,
}
