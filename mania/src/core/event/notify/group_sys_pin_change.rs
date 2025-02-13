use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysPinChangeEvent {
    pub uid: String,
    pub group_uin: Option<u32>,
    pub is_pin: bool,
}
