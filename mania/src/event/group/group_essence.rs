pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupEssenceEvent {
    pub group_uin: u32,
    pub sequence: u32,
    pub random: u32,
    pub is_set: bool,
    pub from_uin: u32,
    pub operator_uin: u32,
}
