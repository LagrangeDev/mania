use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysSpecialTitleEvent {
    pub target_uin: u32,
    pub target_nickname: String,
    pub special_title: String,
    pub special_title_detail_url: String,
    pub group_uin: u32,
}
