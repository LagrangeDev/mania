#[derive(Debug, Default)]
pub struct BotGroupRequest {
    pub group_uin: u32,
    pub invitor_member_uin: Option<u32>,
    pub invitor_member_card: Option<String>,
    pub target_member_uin: u32,
    pub target_member_card: String,
    pub operator_uin: Option<u32>,
    pub operator_name: Option<String>,
    pub sequence: u64,
    pub state: u32,
    pub event_type: u32,
    pub comment: Option<String>,
    pub is_filtered: bool,
}
