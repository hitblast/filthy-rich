#[cfg(unix)]
pub mod ipc;
#[cfg(unix)]
pub mod socket;
#[cfg(unix)]
pub mod utils;

#[cfg(not(unix))]
compile_error!("This crate only supports Unix targets!");
