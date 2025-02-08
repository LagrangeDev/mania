use crate::event::prelude::*;

#[derive(ManiaEvent)]
pub struct BotOnlineEvent {
    pub reason: String,
}

impl Debug for BotOnlineEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[BotOnlineEvent]: reason: {:?}", self.reason)
    }
}
