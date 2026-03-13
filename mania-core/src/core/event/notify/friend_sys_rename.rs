use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct FriendSysRenameEvent {
    pub uid: String,
    pub nickname: String,
}
