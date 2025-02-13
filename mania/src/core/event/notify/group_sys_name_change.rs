use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysNameChangeEvent {
    pub group_uin: u32,
    pub name: String,
}
