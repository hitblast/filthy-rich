//! Core error types module for filthy-rich.
//!
//! The primary error here is [`PresenceError`] which can either be [`PresenceClientError`] or [`PresenceRunnerError`].
//! These two types of errors host their own subset of errors which can be applicable for many different scenarios.
//!
use std::{array::TryFromSliceError, time::SystemTimeError};

use thiserror::Error;
use tokio::{sync::oneshot::error::RecvError, task::JoinError};

/// Details about why the RPC connection was lost.
#[derive(Debug, Clone)]
pub enum DisconnectReason {
    PeerClosed,
    ServerClosed,
    ReadFrameError(String),
    SendFrameError(String),
    SendActivityError(String),
    ClearActivityError(String),
    ClientChannelClosed,
    Unknown,
}

/// Core error type for using with both [`super::PresenceRunner`] and [`super::PresenceClient`].
#[derive(Error, Debug)]
pub enum PresenceError {
    #[error("client error: {0}")]
    ClientError(#[from] PresenceClientError),
    #[error("runner error: {0}")]
    RunnerError(#[from] PresenceRunnerError),
}

/// Points to errors specifically related to [`super::PresenceClient`].
#[derive(Error, Debug)]
pub enum PresenceClientError {
    #[error("failed to send activity")]
    ActivitySendError,
    #[error("failed to receive from runner: {0}")]
    OneShotRecvError(#[from] RecvError),
}

/// Points to errors specifically related to [`super::PresenceRunner`].
#[derive(Error, Debug)]
pub enum PresenceRunnerError {
    #[error("multiple `PresenceRunner::run` calls not allowed")]
    MultipleRun,
    #[error("failed to receive commands")]
    ReceiverError,
    #[error("background task exited before READY")]
    ExitBeforeReady,
    #[error("failed to await on task JoinHandle: {0}")]
    JoinError(#[from] JoinError),
}

#[derive(Error, Debug)]
pub(crate) enum DiscordSockError {
    #[error("could not connect to pipe")]
    PipeConnectionFailed,
    #[error("pipe not found")]
    PipeNotFound,
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to convert to u32 from_le_bytes: {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("failed to parse: {0}")]
    ParseError(#[from] InnerParsingError),
}

#[derive(Error, Debug)]
pub(crate) enum InnerParsingError {
    #[error("failed to serialize to valid JSON: {0}")]
    SerializeFailed(#[from] serde_json::Error),
    #[error("failed to parse system time: {0}")]
    SystemTimeError(#[from] SystemTimeError),
}
