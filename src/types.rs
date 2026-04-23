use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

use crate::utils::filter_none_string;

#[derive(Debug, Serialize)]
pub(crate) struct ActivityCommand {
    cmd: String,
    args: ActivityCommandArgs,
    nonce: String,
}

impl ActivityCommand {
    pub fn new_with(activity: Option<ActivityPayload>) -> Self {
        Self {
            cmd: "SET_ACTIVITY".to_string(),
            args: ActivityCommandArgs {
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

#[derive(Debug, Serialize)]
struct ActivityCommandArgs {
    pid: u32,
    activity: Option<ActivityPayload>,
}

/// Payload that actually gets serialized for setting a rich presence activity.
///
/// Reference: https://docs.discord.com/developers/events/gateway-events#activity-object
#[derive(Debug, Serialize)]
pub(crate) struct ActivityPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub r#type: u8,
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_display_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub timestamps: TimestampPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<AssetsPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<ButtonPayload>>,
}

#[derive(Debug)]
pub(crate) struct AssetsPayload {
    pub large_image: Option<String>,
    pub large_url: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub small_url: Option<String>,
}

// This is redundant as [`ActivityBuilder`] already accepts large_image/small_image fields mandatorily before receiving any of their
// corresponding url/text fields to ensure safety.
impl Serialize for AssetsPayload {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AssetsPayload", 5)?;

        if let Some(v) = &self.large_image {
            state.serialize_field("large_image", v)?;

            if let Some(v) = &self.large_text {
                state.serialize_field("large_text", v)?;
            }
            if let Some(v) = &self.large_url {
                state.serialize_field("large_url", v)?;
            }
        }

        if let Some(v) = &self.small_image {
            state.serialize_field("small_image", v)?;

            if let Some(v) = &self.small_text {
                state.serialize_field("small_text", v)?;
            }
            if let Some(v) = &self.small_url {
                state.serialize_field("small_url", v)?;
            }
        }

        state.end()
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ButtonPayload {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct TimestampPayload {
    pub start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RPCFrame {
    pub cmd: Option<String>,
    pub evt: Option<String>,
    pub data: Option<ReadyData>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DynamicRPCFrame {
    #[allow(unused)]
    pub cmd: Option<String>,
    pub evt: Option<String>,
    #[allow(unused)]
    pub nonce: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug)]
pub(crate) enum IPCCommand {
    SetActivity { activity: Box<Activity> },
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
    // TODO: extend this into a deserializable struct (probably?)
    pub avatar_decoration_data: Option<serde_json::Value>,
    pub bot: bool,
    pub flags: Option<u64>,
    pub premium_type: Option<u64>,
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

/// Represents a Discord Rich Presence activity.
#[derive(Debug, Clone)]
pub struct Activity {
    pub(crate) name: Option<String>,
    pub(crate) activity_type: ActivityType,
    pub(crate) status_display_type: Option<StatusDisplayType>,
    pub(crate) details: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) duration: Option<Duration>,
    pub(crate) large_image: Option<String>,
    pub(crate) large_text: Option<String>,
    pub(crate) large_url: Option<String>,
    pub(crate) small_image: Option<String>,
    pub(crate) small_text: Option<String>,
    pub(crate) small_url: Option<String>,
    pub(crate) buttons: Option<HashMap<String, String>>,
}

impl Activity {
    /// Initializes a new activity builder instance.
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ActivityBuilder {
        ActivityBuilder::default()
    }
}

/// A Rich Presence activity with top text and possibly more attributes.
/// To build an [`Activity`] out of it, use [`ActivityBuilder::build`].
pub struct ActivityBuilder {
    name: Option<String>,
    activity_type: ActivityType,
    status_display_type: Option<StatusDisplayType>,
    details: Option<String>,
    state: Option<String>,
    duration: Option<Duration>,
    large_image: Option<String>,
    large_text: Option<String>,
    large_url: Option<String>,
    small_image: Option<String>,
    small_text: Option<String>,
    small_url: Option<String>,
    buttons: Option<HashMap<String, String>>,
}

impl Default for ActivityBuilder {
    fn default() -> Self {
        Self {
            name: None,
            activity_type: ActivityType::Playing,
            status_display_type: None,
            details: None,
            state: None,
            duration: None,
            large_image: None,
            large_text: None,
            large_url: None,
            small_image: None,
            small_text: None,
            small_url: None,
            buttons: None,
        }
    }
}

impl ActivityBuilder {
    /// Name of the activity.
    pub fn name(mut self, text: impl Into<String>) -> Self {
        self.name = filter_none_string(text);
        self
    }

    /// The type of activity you want to create.
    #[must_use]
    pub fn activity_type(mut self, r#type: ActivityType) -> Self {
        self.activity_type = r#type;
        self
    }

    /// Top text for your activity.
    pub fn details(mut self, text: impl Into<String>) -> Self {
        self.details = filter_none_string(text);
        self
    }

    /// Bottom text for your activity.
    pub fn state(mut self, text: impl Into<String>) -> Self {
        self.state = filter_none_string(text);
        self
    }

    /// The status display type for the activity.
    ///
    /// Errors:
    ///     Can error if `state` is missing and [`StatusDisplayType::State`] has been passed as the type.
    #[must_use]
    pub fn status_display_type(mut self, r#type: StatusDisplayType) -> Self {
        self.status_display_type = Some(r#type);
        self
    }

    /// Countdown duration for your activity.
    #[must_use]
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Add a button to the activity.
    pub fn add_button(mut self, label: impl Into<String>, url: impl Into<String>) -> Self {
        if let Some(btns) = &mut self.buttons {
            btns.insert(label.into(), url.into());
        } else {
            let mut btns: HashMap<String, String> = HashMap::new();
            btns.insert(label.into(), url.into());
            self.buttons = Some(btns);
        };

        self
    }

    /// Large image for your activity (e.g., game icon).
    pub fn large_image(
        mut self,
        key: impl Into<String>,
        text: Option<impl Into<String>>,
        url: Option<impl Into<String>>,
    ) -> Self {
        self.large_image = Some(key.into());
        self.large_text = text.map(|t| t.into());
        self.large_url = url.map(|t| t.into());
        self
    }

    /// Small image for your activity (e.g., status icon).
    pub fn small_image(
        mut self,
        key: impl Into<String>,
        text: Option<impl Into<String>>,
        url: Option<impl Into<String>>,
    ) -> Self {
        self.small_image = Some(key.into());
        self.small_text = text.map(|t| t.into());
        self.small_url = url.map(|t| t.into());
        self
    }

    /// Parses the state of this builder into a usable [`Activity`] for you to pass through [`PresenceClient::set_activity`].
    #[must_use]
    pub fn build(self) -> Activity {
        Activity {
            name: self.name,
            activity_type: self.activity_type,
            status_display_type: self.status_display_type,
            details: self.details,
            state: self.state,
            duration: self.duration,
            large_image: self.large_image,
            large_text: self.large_text,
            large_url: self.large_url,
            small_image: self.small_image,
            small_text: self.small_text,
            small_url: self.small_url,
            buttons: self.buttons,
        }
    }
}
