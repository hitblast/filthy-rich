// SPDX-License-Identifier: MIT

//! This is the API reference page for filthy-rich.
//!
//! ## Getting Started
//!
//! Please refer to either of these two structs for implementing Rich Presence functionality in your app:
//!
//! [`DiscordIPC`] - for async projects.<br>
//! [`DiscordIPCSync`] - for sync projects.
//!
//! ## Examples
//!
//! Library examples can be found in [this directory](https://github.com/hitblast/filthy-rich/tree/master/examples) on GitHub.

mod ipc;
mod client;
mod runner;
pub use types::{Activity, DiscordUser, ReadyData};
pub use client::DiscordIPCClient;
pub use runner::DiscordIPCRunner;
pub use ipc::{DiscordIPC, DiscordIPCSync};

mod socket;
mod types;
