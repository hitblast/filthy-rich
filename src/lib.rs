//! This is the API reference page for filthy-rich.
//!
//! ## Getting Started
//!
//! Please refer to either of these two structs for implementing Rich Presence functionality in your app:
//!
//! [`PresenceRunner`] - core runner for rich presence<br>
//! [`PresenceClient`] - client/messenger instance used for sending activity and closing updates
//!
//! ## Looking for Discord's documentation?
//!
//! The groundwork of this project has been done using the official [Discord RPC documentation](https://docs.discord.com/developers/topics/rpc)
//! page. Do note that not all features are/will be implemented as most of them are not fully implemented in Discord itself on a client basis.
//!
//! ## Examples
//!
//! Library examples can be found in [this directory](https://github.com/hitblast/filthy-rich/tree/master/examples) on GitHub.

mod client;
mod runner;
mod socket;
mod utils;

pub use client::PresenceClient;
pub use runner::PresenceRunner;
pub mod types;
