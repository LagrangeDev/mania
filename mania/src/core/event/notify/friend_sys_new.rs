use mania_macros::DummyEvent;

#[derive(Debug, DummyEvent, Default)]
pub struct FriendSysNewEvent {
    pub from_uid: String,
    pub from_nickname: String,
    pub msg: String,
}
