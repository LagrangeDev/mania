use tokio::sync::watch;

pub mod friend;
pub mod group;
pub mod system;

pub trait ManiaEvent: std::fmt::Debug {}

pub(crate) struct EventDispatcher {
    pub(crate) system: watch::Sender<Option<system::SystemEvent>>,
    pub(crate) friend: watch::Sender<Option<friend::FriendEvent>>,
    pub(crate) group: watch::Sender<Option<group::GroupEvent>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        let (system_tx, _) = watch::channel(None);
        let (friend_tx, _) = watch::channel(None);
        let (group_tx, _) = watch::channel(None);
        Self {
            system: system_tx,
            friend: friend_tx,
            group: group_tx,
        }
    }
}

#[derive(Clone)]
pub struct EventListener {
    pub system: watch::Receiver<Option<system::SystemEvent>>,
    pub friend: watch::Receiver<Option<friend::FriendEvent>>,
    pub group: watch::Receiver<Option<group::GroupEvent>>,
}

impl EventListener {
    pub(crate) fn new(dispatcher: &EventDispatcher) -> Self {
        let system_rx = dispatcher.system.subscribe();
        let friend_rx = dispatcher.friend.subscribe();
        let group_rx = dispatcher.group.subscribe();

        Self {
            system: system_rx,
            friend: friend_rx,
            group: group_rx,
        }
    }
}
