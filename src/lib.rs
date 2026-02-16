// SPDX-License-Identifier: MIT

pub mod ipc;
pub use ipc::{DiscordIPC, DiscordIPCSync};
pub(crate) mod socket;
