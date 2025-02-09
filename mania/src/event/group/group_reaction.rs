pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupReactionEvent {
    pub target_group_uin: u32,
    pub target_sequence: u32,
    pub operator_uin: u32,
    pub is_add: bool,
    pub code: String,
    pub count: u32,
}
