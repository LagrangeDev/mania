pub mod group_message;

#[derive(Debug)]
pub enum GroupEvent {
    GroupMessageEvent(group_message::GroupMessageEvent),
}
