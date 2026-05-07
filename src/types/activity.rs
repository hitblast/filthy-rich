use std::time::Duration;

use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::types::{AssetsPayload, ButtonPayload};

/// A complete Rich Presence activity which can be sent to [`crate::PresenceClient::set_activity`].
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

/// Enum indicating the activity type.
///
/// NOTE: Not all six types are implemented by the Discord client itself, so only the four active
/// ones have been implemented in this enum.
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
    State = 1,
    Details = 2,
}

impl From<StatusDisplayType> for u8 {
    fn from(value: StatusDisplayType) -> Self {
        value as u8
    }
}
