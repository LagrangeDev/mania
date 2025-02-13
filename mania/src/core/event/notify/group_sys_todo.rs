use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysTodoEvent {
    pub group_uin: u32,
    pub operator_uid: String,
}
