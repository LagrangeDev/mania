pub mod bot_online;

#[derive(Debug)]
pub enum BotEvent {
    BotOnlineEvent(bot_online::BotOnlineEvent),
}
