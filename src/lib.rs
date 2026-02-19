// SPDX-License-Identifier: MIT

pub mod ipc;
pub use ipc::{DiscordIPC, DiscordIPCSync};

pub(crate) mod activitytypes;
pub(crate) mod socket;
