use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct FriendSysRequestEvent {
    pub source_uin: u32,
    pub source_uid: String,
    pub message: String,
    pub source: String,
}
