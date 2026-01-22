#[cfg(unix)]
pub mod ipc;
#[cfg(unix)]
pub use ipc::DiscordIPC;
#[cfg(unix)]
mod socket;
#[cfg(unix)]
mod utils;

#[cfg(not(unix))]
compile_error!("This crate only supports Unix targets!");
