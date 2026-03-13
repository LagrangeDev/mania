#![allow(dead_code)] // TODO: remove this after stable
#![feature(default_field_values)]

pub mod business;
pub mod cache;
pub mod connect;
pub mod error;
pub mod event;
mod highway;
mod http;
pub mod operation;
mod ping;
pub mod sign;
pub mod socket;

use crate::business::{Business, BusinessHandle};
use crate::error::ManiaResult;
use crate::sign::default_sign_provider;
pub use mania_core::ClientConfig;
pub use mania_core::core::context::Protocol;
pub use mania_core::core::context::{AppInfo, Context, DeviceInfo};
pub use mania_core::core::key_store::KeyStore;
pub use mania_core::core::session::Session;
pub use mania_core::core::sign::SignProvider;
pub use mania_core::dda;
pub use mania_core::message;
use std::env;
use std::sync::Arc;

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
            session: Arc::new(Session::new(key_store, Some(config.clone()))),
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
