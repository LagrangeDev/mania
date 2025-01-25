mod caching_logic;
mod messaging_logic;
mod wt_logic;

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Display;
use std::future::Future;
use std::io::Result;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::core::context::Context;
use crate::core::event::prelude::*;
use crate::core::event::resolve_event;
use crate::core::packet::SsoPacket;
use crate::core::socket::{self, PacketReceiver, PacketSender};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::sync::{oneshot, Mutex, MutexGuard};

#[derive(Copy, Clone, Debug)]
pub enum LogicFlow {
    InComing,
    OutGoing,
}

impl Display for LogicFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicFlow::InComing => write!(f, ">>> InComing"),
            LogicFlow::OutGoing => write!(f, "<<< OutGoing"),
        }
    }
}

type LogicHandleFn = for<'a> fn(
    &'a mut dyn ServerEvent,
    Arc<Context>,
    LogicFlow,
) -> Pin<Box<dyn Future<Output = &'a dyn ServerEvent> + Send + 'a>>;

pub struct LogicRegistry {
    pub event_type_id_fn: fn() -> TypeId,
    pub event_handle_fn: LogicHandleFn,
}

inventory::collect!(LogicRegistry);

type LogicHandlerMap = HashMap<TypeId, Vec<LogicHandleFn>>;
static LOGIC_MAP: Lazy<LogicHandlerMap> = Lazy::new(|| {
    let mut map = HashMap::new();
    for item in inventory::iter::<LogicRegistry> {
        let tid = (item.event_type_id_fn)();
        map.entry(tid)
            .or_insert_with(Vec::new)
            .push(item.event_handle_fn);
        tracing::debug!(
            "Registered event handler {:?} for type: {:?}",
            item.event_handle_fn,
            tid
        );
    }
    map
});

pub async fn dispatch_logic(
    event: &mut dyn ServerEvent,
    ctx: Arc<Context>,
    flow: LogicFlow,
) -> &dyn ServerEvent {
    let tid = event.as_any().type_id();
    if let Some(fns) = LOGIC_MAP.get(&tid) {
        tracing::debug!("[{}] Found {} handlers for {:?}.", flow, fns.len(), event);
        for handle_fn in fns.iter() {
            handle_fn(event, ctx.to_owned(), flow).await;
        }
    } else {
        tracing::debug!("[{}] No handler found for {:?}", flow, event);
    }
    event
}

pub struct Business {
    addr: SocketAddr,
    receiver: PacketReceiver,
    handle: Arc<BusinessHandle>,
    context: Arc<Context>,
}

impl Business {
    pub async fn new(addr: SocketAddr, context: Arc<Context>) -> Result<Self> {
        let (sender, receiver) = socket::connect(addr).await?;
        let handle = Arc::new(BusinessHandle {
            sender: ArcSwap::new(Arc::new(sender)),
            reconnecting: Mutex::new(()),
            pending_requests: DashMap::new(),
            context: context.clone(),
        });

        Ok(Self {
            addr,
            receiver,
            handle,
            context,
        })
    }

    pub fn handle(&self) -> Arc<BusinessHandle> {
        self.handle.clone()
    }

    // TODO: decouple
    pub async fn spawn(&mut self) {
        let handle_packets = async {
            loop {
                match self.dispatch_packet().await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Error handling packet: {}", e);
                        // FIXME: Non-required reconnections, except for serious errors
                        // self.reconnect().await;
                    }
                }
            }
        };
        tokio::select! {
            _ = handle_packets => {}
        }
    }

    async fn try_reconnect(&mut self) -> Result<()> {
        tracing::info!("Reconnecting to server: {}", self.addr);

        let (sender, receiver) = socket::connect(self.addr).await?;
        self.handle.sender.store(Arc::new(sender));
        self.receiver = receiver;

        todo!("await Collection.Business.WtExchangeLogic.BotOnline(BotOnlineEvent.OnlineReason.Reconnect);")
    }

    async fn reconnect(&mut self) {
        let handle = self.handle.clone();
        let reconnecting = handle.set_reconnecting().await;
        let mut try_interval = Duration::from_secs(1);
        loop {
            match self.try_reconnect().await {
                Ok(_) => break,
                Err(e) => {
                    tracing::error!("Reconnect failed: {}", e);
                    tracing::info!("Retrying in {} seconds", try_interval.as_secs());
                    tokio::time::sleep(try_interval).await;
                    if try_interval < Duration::from_secs(30) {
                        try_interval *= 2;
                    }
                }
            }
        }
        drop(reconnecting);
    }

    async fn dispatch_packet(
        &mut self,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Lagrange.Core.Internal.Context.PacketContext.DispatchPacket
        let packet = self.receiver.recv().await?;
        let packet = SsoPacket::parse(packet, &self.context)?;
        let sequence = packet.sequence();

        let mut event = resolve_event(packet, &self.context).await?;
        // Lagrange.Core.Internal.Context.BusinessContext.HandleIncomingEvent
        // 在 send_event 中的 handle incoming event 合并到这里来 (aka dispatch_logic)
        // GroupSysDecreaseEvent, ... -> Lagrange.Core.Internal.Context.Logic.Implementation.CachingLogic.Incoming
        // KickNTEvent -> Lagrange.Core.Internal.Context.Logic.Implementation.WtExchangeLogic.Incoming
        // PushMessageEvent, ... -> Lagrange.Core.Internal.Context.Logic.Implementation.MessagingLogic.Incoming
        dispatch_logic(&mut *event, self.context.clone(), LogicFlow::InComing).await;
        if let Some((_, tx)) = self.handle.pending_requests.remove(&sequence) {
            tx.send(event).unwrap();
        } else {
            // (actually done)
            // Lagrange.Core.Internal.Context.BusinessContext.HandleServerPacket
            // Lagrange.Core.Internal.Context.BusinessContext.HandleIncomingEvent
            tracing::warn!("unhandled packet: {:?}", event);
        }
        Ok(())
    }
}

pub struct BusinessHandle {
    sender: ArcSwap<PacketSender>,
    reconnecting: Mutex<()>,
    pending_requests: DashMap<u32, oneshot::Sender<Box<dyn ServerEvent>>>,
    context: Arc<Context>,
}

impl BusinessHandle {
    /// Wait if the client is reconnecting.
    async fn wait_reconnecting(&self) {
        drop(self.reconnecting.lock().await);
    }

    /// Set the client to reconnecting state.
    async fn set_reconnecting(&self) -> MutexGuard<'_, ()> {
        self.reconnecting.lock().await
    }

    fn build_sso_packet<T: ClientEvent>(&self, event: &T) -> SsoPacket {
        SsoPacket::new(
            event.packet_type(),
            event.command(),
            event.build(&self.context),
        )
    }

    /// Push a client event to the server, without waiting for a response.
    pub async fn push_event(&self, event: &(impl ClientEvent + ServerEvent)) -> Result<()> {
        let packet = self.build_sso_packet(event);
        self.post_packet(packet).await
    }

    /// Send a client event to the server and wait for the response.
    pub async fn send_event(
        &self,
        event: &mut (impl ClientEvent + ServerEvent),
    ) -> Result<Box<dyn ServerEvent>> {
        // Lagrange.Core.Internal.Context.BusinessContext.HandleOutgoingEvent
        dispatch_logic(
            event as &mut dyn ServerEvent,
            self.context.clone(),
            LogicFlow::OutGoing,
        )
        .await;
        // MultiMsgUploadEvent -> Lagrange.Core.Internal.Context.Logic.Implementation.MessagingLogic.Outgoing
        let packet = self.build_sso_packet(event);
        let events = self.send_packet(packet).await?;
        Ok(events)
    }

    async fn post_packet(&self, packet: SsoPacket) -> Result<()> {
        let packet = packet.build(&self.context);
        self.sender.load().send(packet).await
    }

    async fn send_packet(&self, packet: SsoPacket) -> Result<Box<dyn ServerEvent>> {
        tracing::debug!("sending packet: {:?}", packet);
        let sequence = packet.sequence();
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(sequence, tx);
        self.post_packet(packet).await?;
        let events = rx.await.expect("response not received");
        Ok(events)
    }
}
