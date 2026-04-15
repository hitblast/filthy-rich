//! This is the API reference page for filthy-rich.
//!
//! ## Getting Started
//!
//! Please refer to either of these two structs for implementing Rich Presence functionality in your app:
//!
//! [`PresenceRunner`] - core runner for rich presence<br>
//! [`PresenceClient`] - client/messenger instance used for sending activity and closing updates
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
