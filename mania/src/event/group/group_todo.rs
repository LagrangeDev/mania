pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupTodoEvent {
    pub group_uin: u32,
    pub operator_uin: u32,
}
