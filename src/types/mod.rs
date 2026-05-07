//! Core types related to filthy-rich. Used in conjunction with the core imports.
//!
mod activity;
mod builder;
mod data;
mod payloads;
mod rpc;

pub use activity::{ActivitySpec, ActivityType, StatusDisplayType};
pub use builder::{Activity, ActivityBuilder};
pub use data::{ActivityResponseData, DiscordUser, ReadyData, ServerConfigurationData};
pub use payloads::{ActivityPayload, AssetsPayload, ButtonPayload, TimestampPayload};
pub(crate) use rpc::{
    ActivityCommand, DynamicRPCFrame, IPCCommand, PresenceHandshake, ReadyRPCFrame,
};
