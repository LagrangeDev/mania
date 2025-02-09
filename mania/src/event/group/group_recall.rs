pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupRecallEvent {
    pub group_uin: u32,
    pub author_uin: u32,
    pub operator_uin: u32,
    pub sequence: u32,
    pub time: u32,
    pub random: u32,
    pub tip: String,
}
