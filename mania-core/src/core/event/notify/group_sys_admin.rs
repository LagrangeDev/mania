use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysAdminEvent {
    pub group_uin: u32,
    pub uid: String,
    pub is_promoted: bool,
}
