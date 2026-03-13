mod caching_logic;
mod messaging_logic;
mod wt_logic;

use std::any::TypeId;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::cache::Cache;
use crate::connect::optimum_server;
use crate::event::{EventDispatcher, EventListener};
use crate::highway::{Highway, HighwayError};
use crate::socket::{PacketReceiver, PacketSender};
use crate::{ClientConfig, socket};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use futures::stream::{FuturesUnordered, StreamExt};
use mania_core::core::context::Context;
use mania_core::core::event::{CEParse, ClientEvent, EventError, ServerEvent, resolve_event};
use mania_core::core::packet::SsoPacket;
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::sync::{Mutex, MutexGuard, oneshot};

#[derive(Debug, Error)]
pub enum BusinessError {
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("Invalid server response: {0}")]
    InvalidServerResponse(String),
    #[error("An mania error occurred: {0}")]
    GenericError(Cow<'static, str>),
    #[error("{0}")]
    InternalEventError(#[from] EventError),
    #[error("An mania internal event downcast error occurred")]
    InternalEventDowncastError,
    #[error("An mania internal highway error occurred: {0}")]
    HighWayError(#[from] HighwayError),
}

pub type BusinessResult<T> = Result<T, BusinessError>;

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
    Arc<BusinessHandle>,
    LogicFlow,
) -> Pin<
    Box<dyn Future<Output = Result<&'a dyn ServerEvent, BusinessError>> + Send + 'a>,
>;

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
        tracing::trace!(
            "Registered event handler {:?} for type: {:?}",
            item.event_handle_fn,
            tid
        );
    }
    map
});

pub async fn dispatch_logic(
    event: &mut dyn ServerEvent,
    handle: Arc<BusinessHandle>,
    flow: LogicFlow,
) -> Result<&dyn ServerEvent, BusinessError> {
    let tid = event.as_any().type_id();
    if let Some(fns) = LOGIC_MAP.get(&tid) {
        tracing::trace!("[{}] Found {} handlers for {:?}.", flow, fns.len(), event);
        for handle_fn in fns.iter() {
            handle_fn(event, handle.to_owned(), flow).await?;
        }
    } else {
        tracing::trace!("[{}] No handler found for {:?}", flow, event);
    }
    Ok(event)
}

pub struct Business {
    addr: SocketAddr,
    receiver: PacketReceiver,
    handle: Arc<BusinessHandle>,
}

impl Business {
    pub async fn new(config: Arc<ClientConfig>, context: Arc<Context>) -> BusinessResult<Self> {
        let addr = optimum_server(config.get_optimum_server, config.use_ipv6_network).await?;
        let (sender, receiver) = socket::connect(addr).await?;
        let event_dispatcher = EventDispatcher::new();
        let event_listener = EventListener::new(&event_dispatcher);
        let handle = Arc::new(BusinessHandle {
            sender: ArcSwap::new(Arc::new(sender)),
            reconnecting: Mutex::new(()),
            pending_requests: DashMap::new(),
            context,
            cache: Arc::new(Cache::new(config.cache_mode)), // TODO: construct from context
            event_dispatcher,
            event_listener,
            highway: Arc::new(Highway::default()),
        });

        Ok(Self {
            addr,
            receiver,
            handle,
        })
    }

    pub fn handle(&self) -> Arc<BusinessHandle> {
        self.handle.clone()
    }

    // TODO: decouple
    pub async fn spawn(&mut self) {
        let mut in_flight = FuturesUnordered::new();
        loop {
            tokio::select! {
                result = self.receiver.recv() => {
                    let raw_packet = match result {
                        Ok(packet) => packet,
                        Err(e) => {
                            tracing::error!("Failed to receive raw_packet: {}", e);
                            continue;
                        }
                    };
                    let packet = match SsoPacket::parse(raw_packet, &self.handle.context.session) {
                        Ok(packet) => packet,
                        Err(e) => {
                            tracing::error!("Failed to parse SsoPacket: {}", e);
                            continue;
                        }
                    };
                    tracing::debug!("Incoming packet: {}", packet.command());
                    tracing::trace!("Full: {:?}", packet);
                    let handle = self.handle.clone();
                    in_flight.push(async move {
                        if let Err(e) = handle.dispatch_sso_packet(packet).await {
                            tracing::error!("Unhandled error occurred when handling packet: {:?}", e);
                        }
                    });
                }
                Some(_) = in_flight.next() => {}
            }
        }
    }

    async fn try_reconnect(&mut self) -> BusinessResult<()> {
        tracing::info!("Reconnecting to server: {}", self.addr);

        let (sender, receiver) = socket::connect(self.addr).await?;
        self.handle.sender.store(Arc::new(sender));
        self.receiver = receiver;

        todo!(
            "await Collection.Business.WtExchangeLogic.BotOnline(BotOnlineEvent.OnlineReason.Reconnect);"
        )
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
}

pub struct BusinessHandle {
    sender: ArcSwap<PacketSender>,
    reconnecting: Mutex<()>,
    pending_requests: DashMap<u32, oneshot::Sender<BusinessResult<CEParse>>>,
    pub(crate) context: Arc<Context>,
    pub(crate) cache: Arc<Cache>,
    pub(crate) event_dispatcher: EventDispatcher,
    pub event_listener: EventListener,
    pub(crate) highway: Arc<Highway>,
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

    fn build_sso_packet<T: ClientEvent>(&self, event: &T) -> BusinessResult<SsoPacket> {
        Ok(SsoPacket::new(
            event.packet_type(),
            event.command(),
            event.build(&self.context)?,
        ))
    }

    async fn dispatch_sso_packet(self: &Arc<Self>, packet: SsoPacket) -> BusinessResult<()> {
        let sequence = packet.sequence();
        let result = async {
            let (mut major_event, mut extra_events) = resolve_event(packet, &self.context.session)?;
            let svc = self.clone();
            dispatch_logic(major_event.as_mut(), svc.clone(), LogicFlow::InComing).await?;
            if let Some(ref mut events) = extra_events {
                for event in events.iter_mut() {
                    dispatch_logic(event.as_mut(), svc.clone(), LogicFlow::InComing).await?;
                }
            }
            Ok((major_event, extra_events))
        }
        .await;
        // Lagrange.Core.Internal.Context.BusinessContext.HandleIncomingEvent
        // TODO: timeout auto remove
        match self.pending_requests.remove(&sequence) {
            Some((_, tx)) => tx.send(result).expect("receiver dropped"),
            None => match &result {
                Err(BusinessError::InternalEventError(
                    ie @ (EventError::UnsupportedEvent(_) | EventError::InternalWarning(_)),
                )) => tracing::warn!("{}", ie),
                Err(e) => tracing::error!("Unhandled error occurred: {}", e),
                Ok(_) => {}
            },
        }
        Ok(())
    }

    /// Push a client event to the server, without waiting for a response.
    pub async fn push_event(&self, event: &(impl ClientEvent + ServerEvent)) -> BusinessResult<()> {
        let packet = self.build_sso_packet(event);
        self.post_packet(packet?).await
    }

    /// Send a client event to the server and wait for the response.
    pub async fn send_event(
        self: &Arc<Self>,
        event: &mut (impl ClientEvent + ServerEvent),
    ) -> BusinessResult<CEParse> {
        // Lagrange.Core.Internal.Context.BusinessContext.HandleOutgoingEvent
        dispatch_logic(
            event as &mut dyn ServerEvent,
            self.clone(),
            LogicFlow::OutGoing,
        )
        .await?;
        // MultiMsgUploadEvent -> Lagrange.Core.Internal.Context.Logic.Implementation.MessagingLogic.Outgoing
        let packet = self.build_sso_packet(event);
        let events = self.send_packet(packet?).await?;
        Ok(events)
    }

    async fn post_packet(&self, packet: SsoPacket) -> BusinessResult<()> {
        tracing::debug!("Outgoing packet: {}", packet.command());
        tracing::trace!("Full: {:?}", packet);
        let packet = packet.build(&self.context);
        Ok(self.sender.load().send(packet).await?)
    }

    async fn send_packet(&self, packet: SsoPacket) -> BusinessResult<CEParse> {
        let sequence = packet.sequence();
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(sequence, tx);
        self.post_packet(packet).await?;
        match tokio::time::timeout(Duration::from_secs(15), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => {
                self.pending_requests.remove(&sequence);
                Err(BusinessError::GenericError(
                    "response channel dropped".into(),
                ))
            }
            Err(_) => {
                self.pending_requests.remove(&sequence);
                Err(BusinessError::GenericError("request timed out".into()))
            }
        }
    }
}
