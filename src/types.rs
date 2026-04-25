//! Core types related to filthy-rich. Used in conjunction with the core imports.
//!
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<u8>,
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_display_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_url: Option<String>,
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
        let mut state = serializer.serialize_struct("AssetsPayload", 6)?;

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

/// An iteration of a frame primarily for interpreting received READY events.
#[derive(Debug, Deserialize)]
pub(crate) struct ReadyRPCFrame {
    pub cmd: Option<String>,
    pub evt: Option<String>,
    pub data: Option<ReadyData>,
}

/// An iteration of a frame for using throughout the generic read_frame loop.
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
    SetActivity {
        activity: Box<Activity>,
    },
    ClearActivity,
    Close {
        done: tokio::sync::oneshot::Sender<()>,
    },
}

/// Data received in response from the server after sending a SET_ACTIVITY command.
///
/// Note that this struct doesn't fully cover the schema of the actual response since most of the fields
/// that are found are the same as the actual activity that is sent.
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityResponseData {
    pub application_id: String,
    pub platform: String,
    pub name: String,
    pub metadata: serde_json::Value,
}

/// Data received from READY event.
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
    pub(crate) activity_type: Option<ActivityType>,
    pub(crate) status_display_type: Option<StatusDisplayType>,
    pub(crate) details: Option<String>,
    pub(crate) details_url: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) state_url: Option<String>,
    pub(crate) instance: Option<bool>,
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

impl Default for Activity {
    /// Gives out an empty [`Activity`] with all of the default values. Essentially,
    /// this only shows the name of the app and the elapsed time for the activity on
    /// Discord. Useful when you only need a simple rich presence instance.
    ///
    /// For building a complete activity, using [`Activity::new`] is suggested instead.
    fn default() -> Self {
        Self {
            name: None,
            activity_type: None,
            status_display_type: None,
            details: None,
            details_url: None,
            state: None,
            state_url: None,
            instance: None,
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

/// A Rich Presence activity with top text and possibly more attributes.
/// To build an [`Activity`] out of it, use [`ActivityBuilder::build`].
#[derive(Default)]
pub struct ActivityBuilder {
    name: Option<String>,
    activity_type: Option<ActivityType>,
    status_display_type: Option<StatusDisplayType>,
    instance: Option<bool>,
    details: Option<String>,
    details_url: Option<String>,
    state: Option<String>,
    state_url: Option<String>,
    duration: Option<Duration>,
    large_image: Option<String>,
    large_text: Option<String>,
    large_url: Option<String>,
    small_image: Option<String>,
    small_text: Option<String>,
    small_url: Option<String>,
    buttons: Option<HashMap<String, String>>,
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
        self.activity_type = Some(r#type);
        self
    }

    /// Top text for your activity.
    pub fn details(mut self, text: impl Into<String>) -> Self {
        self.details = filter_none_string(text);
        self
    }

    /// URL for the top text of your activity.
    pub fn details_url(mut self, url: impl Into<String>) -> Self {
        self.details_url = filter_none_string(url);
        self
    }

    /// Bottom text for your activity.
    pub fn state(mut self, text: impl Into<String>) -> Self {
        self.state = filter_none_string(text);
        self
    }

    /// URL for the bottom text of your activity.
    pub fn state_url(mut self, url: impl Into<String>) -> Self {
        self.state_url = filter_none_string(url);
        self
    }

    /// Sets the activity to be an instance.
    #[must_use]
    pub fn set_as_instance(mut self) -> Self {
        self.instance = Some(true);
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
    pub fn large_image(mut self, key: impl Into<String>) -> Self {
        self.large_image = Some(key.into());
        self
    }

    /// Text for the large image of your activity.
    pub fn large_text(mut self, text: impl Into<String>) -> Self {
        self.large_text = Some(text.into());
        self
    }

    /// URL for the large image of your activity.
    pub fn large_url(mut self, url: impl Into<String>) -> Self {
        self.large_url = Some(url.into());
        self
    }

    /// Small image for your activity (e.g., game icon).
    pub fn small_image(mut self, key: impl Into<String>) -> Self {
        self.small_image = Some(key.into());
        self
    }

    /// Text for the small image of your activity.
    pub fn small_text(mut self, text: impl Into<String>) -> Self {
        self.small_text = Some(text.into());
        self
    }

    /// URL for the small image of your activity.
    pub fn small_url(mut self, url: impl Into<String>) -> Self {
        self.small_url = Some(url.into());
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
            details_url: self.details_url,
            state: self.state,
            state_url: self.state_url,
            instance: self.instance,
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
