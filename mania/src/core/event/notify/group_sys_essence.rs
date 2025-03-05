use crate::core::{entity::group_sys_enum::GroupEssenceSetFlag, event::prelude::*};
#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysEssenceEvent {
    pub group_uin: u32,
    pub sequence: u32,
    pub random: u32,
    pub set_flag: GroupEssenceSetFlag,
    pub from_uin: u32,
    pub operator_uin: u32,
}
