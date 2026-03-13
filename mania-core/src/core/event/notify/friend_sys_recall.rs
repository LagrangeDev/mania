use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct FriendSysRecallEvent {
    pub from_uid: String,
    pub client_sequence: u32,
    pub time: u32,
    pub random: u32,
    pub tip: String,
}
