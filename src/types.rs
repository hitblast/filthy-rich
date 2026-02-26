use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct IPCActivityCmd {
    cmd: String,
    args: IPCActivityCmdArgs,
    nonce: String,
}
impl IPCActivityCmd {
    pub fn new_with(activity: Option<ActivityPayload>) -> Self {
        Self {
            cmd: "SET_ACTIVITY".to_string(),
            args: IPCActivityCmdArgs {
                pid: std::process::id(),
                activity,
            },
            nonce: Uuid::new_v4().to_string(),
        }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("Failed to serialize IPC activity command.")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IPCActivityCmdArgs {
    pid: u32,
    activity: Option<ActivityPayload>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) state: Option<String>,
    pub(crate) timestamps: TimestampPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) assets: Option<AssetsPayload>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssetsPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) small_text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimestampPayload {
    pub(crate) start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) end: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct RpcFrame {
    pub cmd: Option<String>,
    pub evt: Option<String>,
    pub data: Option<ReadyData>,
}

#[derive(Debug)]
pub enum IPCCommand {
    SetActivity { activity: Activity },
    ClearActivity,
    Close,
}

/// An object which is passed during READY capture from Discord IPC instance.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadyData {
    pub user: DiscordUser,
}

/// Represents a Discord user.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    pub avatar_decoration_data: Option<serde_json::Value>,
    pub bot: bool,
    pub flags: Option<u64>,
    pub premium_type: Option<u64>,
}

/// Represents a Discord Rich Presence activity.
#[derive(Debug, Clone)]
pub struct Activity {
    pub(crate) details: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) duration: Option<Duration>,
    pub(crate) large_image_key: Option<String>,
    pub(crate) large_image_text: Option<String>,
    pub(crate) small_image_key: Option<String>,
    pub(crate) small_image_text: Option<String>,
}

pub struct ActivityBuilder;

/// A Rich Presence activity with top text and possibly more attributes.
/// [`ActivityWithDetails::build`] needs to be called on it in order to
/// turn it into a proper [`Activity`] instance.
pub struct ActivityWithDetails {
    details: String,
    state: Option<String>,
    duration: Option<Duration>,
    large_image_key: Option<String>,
    large_image_text: Option<String>,
    small_image_key: Option<String>,
    small_image_text: Option<String>,
}

impl Activity {
    pub fn new() -> ActivityBuilder {
        ActivityBuilder
    }

    /// Initializes a Rich Presence activity without any content; useful for small apps.
    pub fn build_empty() -> Self {
        Self {
            details: None,
            state: None,
            duration: None,
            large_image_key: None,
            large_image_text: None,
            small_image_key: None,
            small_image_text: None,
        }
    }
}

impl ActivityBuilder {
    /// Top text for your activity.
    pub fn details(self, details: impl Into<String>) -> ActivityWithDetails {
        ActivityWithDetails {
            details: details.into(),
            state: None,
            duration: None,
            large_image_key: None,
            large_image_text: None,
            small_image_key: None,
            small_image_text: None,
        }
    }
}

impl ActivityWithDetails {
    /// Bottom text for your activity.
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Countdown duration for your activity.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Large image for your activity (e.g., game icon).
    pub fn large_image(mut self, key: impl Into<String>, text: Option<impl Into<String>>) -> Self {
        self.large_image_key = Some(key.into());
        self.large_image_text = text.map(|t| t.into());
        self
    }

    /// Small image for your activity (e.g., status icon).
    pub fn small_image(mut self, key: impl Into<String>, text: Option<impl Into<String>>) -> Self {
        self.small_image_key = Some(key.into());
        self.small_image_text = text.map(|t| t.into());
        self
    }

    /// Parses the state of this builder into a usable [`Activity`] for you to pass through either [`DiscordIPC::set_activity`]
    /// or [`DiscordIPCSync::set_activity`].
    pub fn build(self) -> Activity {
        Activity {
            details: Some(self.details),
            state: self.state,
            duration: self.duration,
            large_image_key: self.large_image_key,
            large_image_text: self.large_image_text,
            small_image_key: self.small_image_key,
            small_image_text: self.small_image_text,
        }
    }
}
