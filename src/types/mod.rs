//! Core types related to filthy-rich. Used in conjunction with the core imports.
//!
mod activity;
mod builder;
mod payloads;
mod rpc;
mod user;

pub use activity::{ActivityResponseData, ActivitySpec, ActivityType, StatusDisplayType};
pub use builder::{Activity, ActivityBuilder};
pub use payloads::{ActivityPayload, AssetsPayload, ButtonPayload, TimestampPayload};
pub(crate) use rpc::{
    ActivityCommand, DynamicRPCFrame, IPCCommand, PresenceHandshake, ReadyRPCFrame,
};
pub use user::{DiscordUser, ReadyData};
