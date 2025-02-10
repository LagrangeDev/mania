pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct BotOnlineEvent {
    pub reason: Option<String>,
}
