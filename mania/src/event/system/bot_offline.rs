pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct BotOfflineEvent {
    pub reason: Option<String>,
}
