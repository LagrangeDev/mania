use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysMemberEnterEvent {
    pub group_uin: u32,
    pub group_member_uin: u32,
    pub style_id: u32,
}
