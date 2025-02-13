pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupSpecialTitleEvent {
    pub target_uin: u32,
    pub target_nickname: String,
    pub special_title: String,
    pub special_title_detail_url: String,
    pub group_uin: u32,
}
