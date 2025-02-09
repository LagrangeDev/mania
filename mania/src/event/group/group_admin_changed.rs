pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupAdminChangedEvent {
    pub group_uin: u32,
    pub admin_uin: u32,
    pub is_promote: bool,
}
