use crate::core::event::prelude::TryFromPrimitive;

#[derive(Debug, Default, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum GroupMemberIncreaseEventType {
    #[default]
    Unknown = 0,
    Approve = 130,
    Invite = 131,
}

#[derive(Debug, Default, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum GroupMemberDecreaseEventType {
    #[default]
    Unknown = 0,
    KickMe = 3,
    Disband = 129,
    Leave = 130,
    Kick = 131,
}

#[derive(Debug, Default, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum GroupEssenceSetFlag {
    #[default]
    Unknown = 0,
    Set = 1,
    Unset = 2,
}
