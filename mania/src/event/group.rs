pub mod group_admin_changed;
pub mod group_essence;
pub mod group_invitation;
pub mod group_invitation_request;
pub mod group_join_request;
pub mod group_member_decrease;
pub mod group_member_enter;
pub mod group_member_increase;
pub mod group_member_mute;
pub mod group_message;
pub mod group_mute;
pub mod group_name_change;
pub mod group_pin_changed;
pub mod group_poke;
pub mod group_reaction;
pub mod group_recall;
pub mod group_special_title;
pub mod group_todo;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // FIXME: do we need spilt or refactoring?
pub enum GroupEvent {
    GroupAdminChanged(group_admin_changed::GroupAdminChangedEvent),
    GroupEssence(group_essence::GroupEssenceEvent),
    GroupInvitation(group_invitation::GroupInvitationEvent),
    GroupInvitationRequest(group_invitation_request::GroupInvitationRequestEvent),
    GroupJoinRequest(group_join_request::GroupJoinRequestEvent),
    GroupMemberDecrease(group_member_decrease::GroupMemberDecreaseEvent),
    GroupMemberEnter(group_member_enter::GroupMemberEnterEvent),
    GroupMemberIncrease(group_member_increase::GroupMemberIncreaseEvent),
    GroupMemberMute(group_member_mute::GroupMemberMuteEvent),
    GroupMessage(group_message::GroupMessageEvent), // FIXME: clippy warn: at least 320 bytes
    GroupMute(group_mute::GroupMuteEvent),
    GroupNameChange(group_name_change::GroupNameChangeEvent),
    GroupPinChanged(group_pin_changed::PinChangedEvent),
    GroupPoke(group_poke::GroupPokeEvent), // FIXME: clippy warn: at least 88 bytes
    GroupReaction(group_reaction::GroupReactionEvent),
    GroupRecall(group_recall::GroupRecallEvent),
    GroupTodo(group_todo::GroupTodoEvent),
    GroupSpecialTitle(group_special_title::GroupSpecialTitleEvent),
}
