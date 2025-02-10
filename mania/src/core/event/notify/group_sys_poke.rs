use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysPokeEvent {
    pub group_uin: u32,
    pub operator_uin: u32,
    pub target_uin: u32,
    pub action: String,
    pub suffix: String,
    pub action_img_url: String,
}
