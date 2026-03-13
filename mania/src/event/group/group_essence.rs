use mania_core::core::entity::group_sys_enum::GroupEssenceSetFlag;
use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupEssenceEvent {
    pub group_uin: u32,
    pub sequence: u32,
    pub random: u32,
    pub is_set: GroupEssenceSetFlag,
    pub from_uin: u32,
    pub operator_uin: u32,
}
