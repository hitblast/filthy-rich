//! Core error types module for filthy-rich.
//!
//! The primary error here is [`PresenceError`] which can either be [`PresenceClientError`] or [`PresenceRunnerError`].
//! These two types of errors host their own subset of errors which can be applicable for many different scenarios.
//!
use std::{array::TryFromSliceError, time::SystemTimeError};

use thiserror::Error;
use tokio::{sync::oneshot::error::RecvError, task::JoinError};

/// Error which happens if building a [`crate::types::ActivitySpec`] fails via [`crate::types::ActivityBuilder::build`].
#[derive(Error, Debug)]
pub enum ActivitySpecBuildError {
    #[error("image assets are provided without the image itself")]
    ImageAssetsTooEarly,
    #[error("status display points to {0} but {0} is not set")]
    StatusDisplayElementMissing(&'static str),
    #[error("{0}_url provided but {0} itself is missing")]
    ElementURLProvidedEarly(&'static str),
}

/// Details about why the RPC connection was lost.
#[derive(Debug, Clone)]
pub enum DisconnectReason {
    PeerClosed,
    ServerClosed,
    OldRelicComputer(String),
    ReadFrameError(String),
    SendFrameError(String),
    SendActivityError(String),
    ClearActivityError(String),
    ClientChannelClosed,
    Unknown,
}

/// Core convenience error type for usage with all types of `filthy-rich` operations, including [`crate::PresenceRunner`] and [`crate::PresenceClient`].
#[derive(Error, Debug)]
pub enum PresenceError {
    #[error("client error: {0}")]
    ClientError(#[from] PresenceClientError),
    #[error("runner error: {0}")]
    RunnerError(#[from] PresenceRunnerError),
    #[error("activity spec build error: {0}")]
    BuildError(#[from] ActivitySpecBuildError),
}

/// Points to errors specifically related to [`crate::PresenceClient`].
#[derive(Error, Debug)]
pub enum PresenceClientError {
    #[error("failed to send activity")]
    ActivitySendError,
    #[error("failed to receive from runner: {0}")]
    OneShotRecvError(#[from] RecvError),
}

/// Points to errors specifically related to [`crate::PresenceRunner`].
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
    #[error("payload size {size} exceeds maximum allowed size {max}")]
    PayloadTooLarge { size: usize, max: usize },
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
