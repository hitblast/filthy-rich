//! Core types related to filthy-rich. Used in conjunction with the core imports.
//!
mod activity;
mod builder;
pub mod data;
pub mod payloads;
mod rpc;

pub use activity::{ActivitySpec, ActivityType, StatusDisplayType};
pub use builder::{Activity, ActivityBuilder};
pub(crate) use rpc::{
    ActivityCommand, DynamicRPCFrame, IPCCommand, PresenceHandshake, ReadyRPCFrame,
};
