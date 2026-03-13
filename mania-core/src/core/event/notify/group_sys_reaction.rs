use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysReactionEvent {
    pub target_group_uin: u32,
    pub target_sequence: u32,
    pub operator_uid: String,
    pub is_add: bool,
    pub code: String,
    pub count: u32,
}
