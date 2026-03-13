#![allow(dead_code)]
#![feature(default_field_values)]

pub mod core;
pub mod entity;
pub mod message;
pub mod utility;

use crate::core::context::Context;
use crate::core::context::Protocol;
use crate::core::sign::SignProvider;
use crate::entity::bot_group_member::FetchGroupMemberStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    Full,
    Half,
    None,
}

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
