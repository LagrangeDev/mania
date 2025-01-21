use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use dashmap::DashMap;
use tokio::sync::{oneshot, Mutex, MutexGuard};

use crate::context::Context;
use crate::event::{resolve_events, ClientEvent, ServerEvent};
use crate::packet::SsoPacket;
use crate::socket::{self, PacketReceiver, PacketSender};

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

        let events = resolve_events(packet, &self.context).await?;
        // TODO: Lagrange.Core.Internal.Context.BusinessContext.HandleIncomingEvent
        // 在 send_event 中的 handle incoming event 合并到这里来
        // GroupSysDecreaseEvent, ... -> Lagrange.Core.Internal.Context.Logic.Implementation.CachingLogic.Incoming
        // KickNTEvent -> Lagrange.Core.Internal.Context.Logic.Implementation.WtExchangeLogic.Incoming
        // PushMessageEvent, ... -> Lagrange.Core.Internal.Context.Logic.Implementation.MessagingLogic.Incoming
        if let Some((_, tx)) = self.handle.pending_requests.remove(&sequence) {
            tx.send(events).unwrap();
        } else {
            // Lagrange.Core.Internal.Context.BusinessContext.HandleServerPacket
            // todo!("Lagrange.Core.Internal.Context.BusinessContext.HandleIncomingEvent")
            tracing::warn!("unhandled packet: {:?}", events);
        }
        Ok(())
    }
}

pub struct BusinessHandle {
    sender: ArcSwap<PacketSender>,
    reconnecting: Mutex<()>,
    pending_requests: DashMap<u32, oneshot::Sender<Vec<Box<dyn ServerEvent>>>>,
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

    /// Push a client event to the server, without waiting for a response.
    pub async fn push_event(&self, event: &impl ClientEvent) -> Result<()> {
        for packet in event.build_sso_packets(&self.context).await {
            self.post_packet(packet).await?;
        }
        Ok(())
    }

    /// Send a client event to the server and wait for the response.
    pub async fn send_event(&self, event: &impl ClientEvent) -> Result<Vec<Box<dyn ServerEvent>>> {
        // TODO: Lagrange.Core.Internal.Context.BusinessContext.HandleOutgoingEvent
        // MultiMsgUploadEvent -> Lagrange.Core.Internal.Context.Logic.Implementation.MessagingLogic.Outgoing
        let mut result = vec![];
        for packet in event.build_sso_packets(&self.context).await {
            let events = self.send_packet(packet).await?;
            result.extend(events);
        }
        Ok(result)
    }

    async fn post_packet(&self, packet: SsoPacket) -> Result<()> {
        let packet = packet.build(&self.context);
        self.sender.load().send(packet).await
    }

    async fn send_packet(&self, packet: SsoPacket) -> Result<Vec<Box<dyn ServerEvent>>> {
        tracing::debug!("sending packet: {:?}", packet);
        let sequence = packet.sequence();
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(sequence, tx);

        self.post_packet(packet).await?;

        let events = rx.await.expect("response not received");
        Ok(events)
    }
}
