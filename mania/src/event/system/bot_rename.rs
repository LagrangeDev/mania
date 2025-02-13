pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct BotRenameEvent {
    pub nickname: String,
}
