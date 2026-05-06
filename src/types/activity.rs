use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::types::{ActivityPayload, AssetsPayload, ButtonPayload};

/// A complete Rich Presence activity which can be sent to [`super::PresenceClient::set_activity`].
#[derive(Default, Clone)]
pub struct ActivitySpec {
    pub(crate) name: Option<String>,
    pub(crate) r#type: Option<ActivityType>,
    pub(crate) instance: Option<bool>,
    pub(crate) status_display_type: Option<StatusDisplayType>,
    pub(crate) details: Option<String>,
    pub(crate) details_url: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) state_url: Option<String>,
    pub(crate) assets: Option<AssetsPayload>,
    pub(crate) buttons: Option<Vec<ButtonPayload>>,
    pub(crate) duration: Option<Duration>,
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
    metadata: Option<Value>,
    #[serde(flatten)]
    activity: ActivityPayload,
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
    pub fn metadata(&self) -> Option<&Value> {
        self.metadata.as_ref()
    }

    #[must_use]
    pub fn activity(&self) -> &ActivityPayload {
        &self.activity
    }
}

/// Enum indicating the activity type.
#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr, Copy)]
#[serde(into = "u8", try_from = "u8")]
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
#[derive(Clone, Debug, Eq, PartialEq, Copy, Serialize_repr, Deserialize_repr)]
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
