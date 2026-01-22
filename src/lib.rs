pub mod ipc;
pub mod socket;
pub mod utils;

#[cfg(not(unix))]
compile_error!("This crate only supports Unix targets!");
