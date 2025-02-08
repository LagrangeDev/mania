use tokio::sync::watch;

pub mod bot;
pub mod friend;
pub mod group;

pub trait ManiaEvent: std::fmt::Debug {}

pub(crate) struct EventDispatcher {
    pub(crate) bot: watch::Sender<Option<bot::BotEvent>>,
    pub(crate) friend: watch::Sender<Option<friend::FriendEvent>>,
    pub(crate) group: watch::Sender<Option<group::GroupEvent>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        let (bot_tx, _) = watch::channel(None);
        let (friend_tx, _) = watch::channel(None);
        let (group_tx, _) = watch::channel(None);
        Self {
            bot: bot_tx,
            friend: friend_tx,
            group: group_tx,
        }
    }
}

#[derive(Clone)]
pub struct EventListener {
    pub bot: watch::Receiver<Option<bot::BotEvent>>,
    pub friend: watch::Receiver<Option<friend::FriendEvent>>,
    pub group: watch::Receiver<Option<group::GroupEvent>>,
}

impl EventListener {
    pub(crate) fn new(dispatcher: &EventDispatcher) -> Self {
        let bot_rx = dispatcher.bot.subscribe();
        let friend_rx = dispatcher.friend.subscribe();
        let group_rx = dispatcher.group.subscribe();

        Self {
            bot: bot_rx,
            friend: friend_rx,
            group: group_rx,
        }
    }
}

mod prelude {
    pub use mania_macros::ManiaEvent;
    pub use std::fmt::{Debug, Formatter, Result as FmtResult};
}
