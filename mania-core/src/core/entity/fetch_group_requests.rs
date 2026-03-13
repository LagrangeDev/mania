#[derive(Debug, Default)]
pub struct FetchGroupRequests {
    pub group_uin: u32,
    pub invitor_member_uid: Option<String>,
    pub invitor_member_card: Option<String>,
    pub target_member_uid: String,
    pub target_member_card: String,
    pub operator_uid: Option<String>,
    pub operator_name: Option<String>,
    pub sequence: u64,
    pub state: u32,
    pub event_type: u32,
    pub comment: Option<String>,
    pub is_filtered: bool,
}
