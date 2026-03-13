use crate::core::event::prelude::*;

#[derive(Debug, DummyEvent, Default)]
pub struct BotSysRenameEvent {
    pub nickname: String,
}
