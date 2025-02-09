pub use mania_macros::ManiaEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatType {
    Friend,
    Group,
    Service,
}

#[derive(ManiaEvent)]
pub struct PinChangedEvent {
    pub chat_type: ChatType,
    pub uin: u32,
    pub is_pin: bool,
}
