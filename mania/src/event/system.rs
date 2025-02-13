pub mod bot_offline;
pub mod bot_online;
pub mod bot_rename;
pub mod temp_message;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // FIXME: do we need spilt or refactoring?
pub enum SystemEvent {
    BotOnlineEvent(bot_online::BotOnlineEvent), // FIXME: clippy warn: at least 24 bytes
    BotOfflineEvent(bot_offline::BotOfflineEvent),
    TempMessageEvent(temp_message::TempMessageEvent), // FIXME: clippy warn: at least 320 bytes
    BotRenameEvent(bot_rename::BotRenameEvent),
}
