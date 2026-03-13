use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct GroupSysRequestInvitationEvent {
    pub group_uin: u32,
    pub target_uid: String,
    pub invitor_uid: String,
}
