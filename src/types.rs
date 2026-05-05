//! Core types related to filthy-rich. Used in conjunction with the core imports.
//!
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

use crate::{
    errors::InnerParsingError,
    utils::{filter_none_string, get_current_timestamp},
};

#[derive(Serialize)]
pub(crate) struct ActivityCommand {
    cmd: &'static str,
    args: ActivityCommandArgs,
    nonce: String,
}

impl ActivityCommand {
    pub fn new_with(activity: Option<ActivityPayload>) -> Self {
        Self {
            cmd: "SET_ACTIVITY",
            args: ActivityCommandArgs {
                pid: std::process::id(),
                activity,
            },
            nonce: Uuid::new_v4().to_string(),
        }
    }

    pub fn to_json(&self) -> Result<String, InnerParsingError> {
        serde_json::to_string(self).map_err(InnerParsingError::SerializeFailed)
    }
}

#[derive(Serialize)]
struct ActivityCommandArgs {
    pid: u32,
    activity: Option<ActivityPayload>,
}

#[derive(Serialize)]
pub(crate) struct PresenceHandshake<'a> {
    pub v: u8,
    pub client_id: &'a str,
}

/// A complete Rich Presence activity which can be sent to [`super::PresenceClient::set_activity`].
#[derive(Default, Clone)]
pub struct SendableActivity {
    name: Option<String>,
    r#type: Option<u8>,
    instance: Option<bool>,
    status_display_type: Option<u8>,
    details: Option<String>,
    details_url: Option<String>,
    state: Option<String>,
    state_url: Option<String>,
    assets: Option<AssetsPayload>,
    buttons: Option<Vec<ButtonPayload>>,
    duration: Option<Duration>,
}

#[derive(Serialize)]
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

impl ActivityPayload {
    pub fn create(value: SendableActivity, session_start: u64) -> Result<Self, InnerParsingError> {
        let current_t = get_current_timestamp()?;
        let end_timestamp = value.duration.map(|d| current_t + d.as_secs());

        Ok(Self {
            name: value.name,
            r#type: value.r#type,
            created_at: current_t,
            instance: value.instance,
            status_display_type: value.status_display_type,
            details: value.details,
            details_url: value.details_url,
            state: value.state,
            state_url: value.state_url,
            timestamps: TimestampPayload {
                start: session_start,
                end: end_timestamp,
            },
            assets: value.assets,
            buttons: value.buttons,
        })
    }
}

#[derive(Clone)]
pub(crate) struct AssetsPayload {
    pub large_image: Option<String>,
    pub large_url: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub small_url: Option<String>,
}

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

#[derive(Serialize, Clone)]
pub(crate) struct ButtonPayload {
    pub label: String,
    pub url: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct TimestampPayload {
    pub start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[derive(Deserialize)]
pub(crate) struct ReadyRPCFrame {
    pub cmd: Option<String>,
    pub evt: Option<String>,
    pub data: Option<ReadyData>,
}

#[derive(Deserialize)]
pub(crate) struct DynamicRPCFrame {
    #[allow(unused)]
    pub cmd: Option<String>,
    pub evt: Option<String>,
    #[allow(unused)]
    pub nonce: Option<String>,
    pub data: Option<Value>,
}

pub(crate) enum IPCCommand {
    SetActivity {
        activity: Box<SendableActivity>,
    },
    ClearActivity,
    Close {
        done_tx: tokio::sync::oneshot::Sender<()>,
    },
}

/// Data received in response from the server after sending a SET_ACTIVITY command.
///
/// Note that this struct doesn't fully cover the schema of the actual response since most of the fields
/// that are found are the same as the actual activity that is sent.
///
/// More importantly, open-source implementations of RPC (e.g. arRPC) have different response styles so
/// the actual output of this struct may vary depending on what client you are using.
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityResponseData {
    application_id: Option<String>,
    platform: Option<String>,
    name: Option<String>,
    metadata: Option<Value>,
}

impl ActivityResponseData {
    #[must_use]
    pub fn application_id(&self) -> Option<&str> {
        self.application_id.as_deref()
    }

    #[must_use]
    pub fn platform(&self) -> Option<&str> {
        self.platform.as_deref()
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub fn metadata(&self) -> Option<&Value> {
        self.metadata.as_ref()
    }
}

/// Data received from READY event.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadyData {
    user: DiscordUser,
}

impl ReadyData {
    #[must_use]
    pub fn user(&self) -> &DiscordUser {
        &self.user
    }
}

/// Represents a Discord user.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordUser {
    id: String,
    username: String,
    global_name: Option<String>,
    discriminator: Option<String>,
    avatar: Option<String>,
    avatar_decoration_data: Option<Value>,
    bot: bool,
    flags: Option<u64>,
    premium_type: Option<u64>,
}

impl DiscordUser {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    #[must_use]
    pub fn global_name(&self) -> Option<&str> {
        self.global_name.as_deref()
    }

    #[must_use]
    pub fn discriminator(&self) -> Option<&str> {
        self.discriminator.as_deref()
    }

    #[must_use]
    pub fn avatar(&self) -> Option<&str> {
        self.avatar.as_deref()
    }

    #[must_use]
    pub fn avatar_decoration_data(&self) -> Option<&Value> {
        self.avatar_decoration_data.as_ref()
    }

    #[must_use]
    pub fn bot(&self) -> bool {
        self.bot
    }

    #[must_use]
    pub fn flags(&self) -> Option<u64> {
        self.flags
    }

    #[must_use]
    pub fn premium_type(&self) -> Option<u64> {
        self.premium_type
    }
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

/// Represents a Discord Rich Presence activity which is yet to be built. To start building it into a usable [`SendableActivity`],
/// initialize a new [`ActivityBuilder`] with [`Activity::new`].
#[derive(Debug, Clone)]
pub struct Activity;

impl Activity {
    /// Initializes a new activity builder instance.
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ActivityBuilder {
        ActivityBuilder::default()
    }

    /// Gives out an empty but usable [`SendableActivity`]. Essentially,
    /// this only shows the name of the app and the elapsed time for the activity on
    /// Discord. Useful when you only need a simple rich presence instance.
    ///
    /// For building a complete activity, using [`Activity::new`] is suggested instead.
    #[must_use]
    pub fn empty() -> SendableActivity {
        SendableActivity {
            name: None,
            r#type: None,
            status_display_type: None,
            details: None,
            details_url: None,
            state: None,
            state_url: None,
            instance: None,
            assets: None,
            buttons: None,
            duration: None,
        }
    }
}

/// A builder for a Rich Presence activity.
/// To build a [`SendableActivity`] out of it, use [`ActivityBuilder::build`].
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

    /// Adds a button to the activity.
    ///
    /// NOTE: The Discord desktop client may behave in such a way that the buttons may only be visible from anyone but the
    /// connected user's side. This is a wonky feature and must be used with care.
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

    /// Parses the state of this builder into a usable [`SendableActivity`] for you to pass through [`super::PresenceClient::set_activity`].
    #[must_use]
    pub fn build(self) -> SendableActivity {
        SendableActivity {
            name: self.name,
            r#type: self.activity_type.map(|f| f.into()),
            status_display_type: self.status_display_type.map(|f| f.into()),
            details: self.details,
            details_url: self.details_url,
            state: self.state,
            state_url: self.state_url,
            instance: self.instance,
            assets: Some(AssetsPayload {
                large_image: self.large_image,
                large_url: self.large_url,
                large_text: self.large_text,
                small_image: self.small_image,
                small_text: self.small_text,
                small_url: self.small_url,
            }),
            buttons: self.buttons.map(|btns| {
                btns.into_iter()
                    .map(|f| ButtonPayload {
                        label: f.0,
                        url: f.1,
                    })
                    .collect()
            }),
            duration: self.duration,
        }
    }
}
