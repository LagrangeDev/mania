use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysInviteEvent {
    pub group_uin: u32,
    pub invitor_uid: String,
}
