pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupNameChangeEvent {
    pub group_uin: u32,
    pub name: String,
}
