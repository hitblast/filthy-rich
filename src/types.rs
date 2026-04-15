use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct IPCActivityCmd {
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
struct IPCActivityCmdArgs {
    pid: u32,
    activity: Option<ActivityPayload>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ActivityPayload {
    pub activity_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_display_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub timestamps: TimestampPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<AssetsPayload>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct AssetsPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TimestampPayload {
    pub start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RpcFrame {
    pub cmd: Option<String>,
    pub evt: Option<String>,
    pub data: Option<ReadyData>,
}

#[derive(Debug)]
pub(crate) enum IPCCommand {
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
    pub(crate) activity_type: ActivityType,
    pub(crate) status_display_type: Option<StatusDisplayType>,
    pub(crate) details: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) duration: Option<Duration>,
    pub(crate) large_image_key: Option<String>,
    pub(crate) large_image_text: Option<String>,
    pub(crate) small_image_key: Option<String>,
    pub(crate) small_image_text: Option<String>,
}

/// The initial builder for [`Activity`].
pub struct ActivityBuilder {
    activity_type: ActivityType,
}

/// Enum indicating the activity type.
#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum ActivityType {
    Playing = 0,
    Listening = 2,
    Watching = 3,
    Competing = 5,
}

impl From<ActivityType> for u8 {
    fn from(value: ActivityType) -> Self {
        value as u8
    }
}

/// Enum indicating which mode to use for indicating the status of an activity.
#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum StatusDisplayType {
    Name = 0,
    Details = 2,
    State = 1,
}

impl From<StatusDisplayType> for u8 {
    fn from(value: StatusDisplayType) -> Self {
        value as u8
    }
}

impl Activity {
    /// Initializes a new activity builder instance.
    pub fn new() -> ActivityBuilder {
        ActivityBuilder {
            activity_type: ActivityType::Playing,
        }
    }

    /// Initializes a new activity builder instance with a custom activity type.
    pub fn new_with_type(r#type: ActivityType) -> ActivityBuilder {
        ActivityBuilder {
            activity_type: r#type,
        }
    }
}

impl ActivityBuilder {
    /// Top text for your activity.
    pub fn details(self, details: impl Into<String>) -> ActivityBuilderWithDetails {
        ActivityBuilderWithDetails {
            activity_type: self.activity_type,
            status_display_type: None,
            details: details.into(),
            state: None,
            duration: None,
            large_image_key: None,
            large_image_text: None,
            small_image_key: None,
            small_image_text: None,
        }
    }

    /// Builds an [`Activity`] with empty fields, except the activity type, which can either
    /// be developer-defined or the default one.
    pub fn build_empty(self) -> Activity {
        Activity {
            activity_type: self.activity_type,
            status_display_type: None,
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

/// A Rich Presence activity with top text and possibly more attributes.
/// [`ActivityBuilderWithDetails::build`] needs to be called on it in order to
/// turn it into a proper [`Activity`] instance.
pub struct ActivityBuilderWithDetails {
    activity_type: ActivityType,
    status_display_type: Option<StatusDisplayType>,
    details: String,
    state: Option<String>,
    duration: Option<Duration>,
    large_image_key: Option<String>,
    large_image_text: Option<String>,
    small_image_key: Option<String>,
    small_image_text: Option<String>,
}

impl ActivityBuilderWithDetails {
    /// Bottom text for your activity.
    pub fn state(mut self, state: impl Into<String>) -> Self {
        let state: String = state.into();

        if !state.is_empty() {
            self.state = Some(state);
        }

        self
    }

    /// The status display type for the activity.
    pub fn status_display_type(mut self, r#type: StatusDisplayType) -> Self {
        self.status_display_type = Some(r#type);
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
            activity_type: self.activity_type,
            status_display_type: self.status_display_type,
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
