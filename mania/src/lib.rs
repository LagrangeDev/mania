#![allow(dead_code)] // TODO: remove this after stable
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(default_field_values)]
#![feature(result_flattening)]

mod core;
pub mod entity;
pub mod event;
pub mod message;
pub mod utility;

use crate::core::business::{Business, BusinessHandle};
pub use crate::core::cache::CacheMode;
use crate::core::context::Protocol;
pub use crate::core::context::{AppInfo, Context, DeviceInfo};
pub use crate::core::error::{ManiaError, ManiaResult};
pub use crate::core::key_store::KeyStore;
use crate::core::session::Session;
use crate::core::sign::{SignProvider, default_sign_provider};
use crate::entity::bot_group_member::FetchGroupMemberStrategy;
use std::env;
use std::sync::Arc;

/// Configuration for the client
pub struct ClientConfig {
    /// The protocol for the client, default is Linux
    pub protocol: Protocol,
    /// Auto reconnect to server when disconnected
    pub auto_reconnect: bool,
    /// Use the IPv6 to connect to server, only if your network support IPv6
    pub use_ipv6_network: bool,
    /// Get optimum server from Tencent MSF server, set to false to use hardcode server
    pub get_optimum_server: bool,
    /// Custom Sign Provider
    pub sign_provider: Option<Box<dyn SignProvider>>,
    /// The maximum size of the highway block in byte, max 1MB (1024 * 1024 byte)
    pub highway_chuck_size: usize,
    /// Highway uploading concurrency, if the image failed to send, set this to 1
    pub highway_concurrency: usize,
    /// Cache mode for the client
    pub cache_mode: CacheMode,
    /// The strategy for fetching `BotGroupMember`
    /// Setting it to `Simple` can avoid fetching all group members at the cost of losing some fields
    /// See `BotGroupMember` for more information
    pub fetch_group_member_strategy: FetchGroupMemberStrategy,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            protocol: Protocol::Linux,
            auto_reconnect: true,
            use_ipv6_network: false,
            get_optimum_server: true,
            sign_provider: None,
            highway_chuck_size: 1024 * 1024,
            highway_concurrency: 4,
            cache_mode: CacheMode::Half,
            fetch_group_member_strategy: FetchGroupMemberStrategy::Simple,
        }
    }
}

pub struct Client {
    business: Business,
    handle: ClientHandle,
}

impl Client {
    pub async fn new(
        mut config: ClientConfig,
        device: DeviceInfo,
        key_store: KeyStore,
    ) -> ManiaResult<Self> {
        let sign_provider = config.sign_provider.take();
        let config = Arc::new(config);
        let app_info = AppInfo::get(config.protocol);
        let context = Context {
            app_info,
            device,
            key_store,
            config: config.clone(),
            sign_provider: sign_provider.unwrap_or_else(|| {
                default_sign_provider(
                    config.protocol,
                    env::var("MANIA_LINUX_SIGN_URL")
                        .ok()
                        .map(Some)
                        .unwrap_or_else(|| {
                            tracing::warn!("MANIA_LINUX_SIGN_URL not set, login maybe fail!");
                            None
                        }),
                )
            }),
            crypto: Default::default(),
            session: Session::new(),
        };
        let context = Arc::new(context);
        let business = Business::new(config, context.clone()).await?;
        let handle = ClientHandle {
            business: business.handle(),
            context,
        };

        Ok(Self { business, handle })
    }

    pub fn handle(&self) -> ClientHandle {
        self.handle.clone()
    }

    pub async fn spawn(&mut self) {
        self.business.spawn().await;
    }
}

#[derive(Clone)]
pub struct ClientHandle {
    business: Arc<BusinessHandle>,
    context: Arc<Context>,
}

// TODO: (maybe) refactor structure to more user-friendly api?
impl ClientHandle {
    pub fn operator(&self) -> Arc<BusinessHandle> {
        self.business.clone()
    }
}
