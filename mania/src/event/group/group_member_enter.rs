pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupMemberEnterEvent {
    pub group_uin: u32,
    pub group_member_uin: u32,
    pub style_id: u32,
}
