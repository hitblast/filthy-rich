// SPDX-License-Identifier: MIT

#[cfg(unix)]
pub mod ipc;
#[cfg(unix)]
pub use ipc::DiscordIPC;
#[cfg(unix)]
pub(crate) mod socket;
#[cfg(unix)]
pub(crate) mod utils;

#[cfg(not(unix))]
compile_error!("This crate only supports Unix targets!");
