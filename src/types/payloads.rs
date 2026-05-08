//! Payload types which are used for immutable sends and receives. These do not need to be imported separately as the
//! values are often already supplied to you via closures, and are clonable / can be debugged.
//!
use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::{
    ds,
    errors::InnerParsingError,
    types::{ActivitySpec, ActivityType, StatusDisplayType},
    utils::get_current_timestamp,
};

/// The assets payload for the activity. Usually contains the large image and the small image, and their respective
/// subsets of information (i.e. URL and text).
#[derive(Debug, Deserialize, Clone)]
pub struct AssetsPayload {
    pub(crate) large_image: Option<String>,
    pub(crate) large_url: Option<String>,
    pub(crate) large_text: Option<String>,
    pub(crate) small_image: Option<String>,
    pub(crate) small_text: Option<String>,
    pub(crate) small_url: Option<String>,
}

impl AssetsPayload {
    ds!(large_image, "The key for the large image.");
    ds!(large_url, "The URL for the large image.");
    ds!(large_text, "The hover text for the large image.");
    ds!(small_image, "The key for the small image.");
    ds!(small_url, "The URL for the small image.");
    ds!(small_text, "The hover text for the small image.");
}

// redundant: already filtered during [`crate::types::ActivityBuilder::build`]
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

/// Represents a button of the activity.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonPayload {
    pub(crate) label: String,
    pub(crate) url: String,
}

impl ButtonPayload {
    /// The label for the button.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_ref()
    }
    /// The URL the button redirects to when clicked on.
    #[must_use]
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
}

/// The timestamps (calculated with `UNIX_EPOCH`) for the activity.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimestampPayload {
    start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<u64>,
}

impl TimestampPayload {
    /// The starting timestamp for the activity.
    #[must_use]
    pub fn start(&self) -> u64 {
        self.start
    }
    /// The end timestamp for the activity.
    #[must_use]
    pub fn end(&self) -> Option<u64> {
        self.end
    }
}

/// Represents an immutable activity which has been sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<ActivityType>,
    created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_display_type: Option<StatusDisplayType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_url: Option<String>,
    timestamps: TimestampPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    assets: Option<AssetsPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    buttons: Option<Vec<ButtonPayload>>,
}

impl ActivityPayload {
    ds!(name, "The name for the activity.");

    /// The type of the activity.
    #[must_use]
    pub fn activity_type(&self) -> Option<ActivityType> {
        self.r#type
    }
    /// When the activity was created.
    #[must_use]
    pub fn created_at(&self) -> u64 {
        self.created_at
    }
    /// Whether or not the activity is an instance.
    #[must_use]
    pub fn instance(&self) -> Option<bool> {
        self.instance
    }
    /// Which element the activity displays as the status (primary text on members banner).
    #[must_use]
    pub fn status_display_type(&self) -> Option<StatusDisplayType> {
        self.status_display_type
    }

    ds!(details, "The details (top text) for the activity.");
    ds!(
        details_url,
        "The URL which the details field redirects to when clicked on."
    );
    ds!(
        state,
        "The state (usually the bottom text but could be on top) for the activity."
    );
    ds!(
        state_url,
        "The URL which the state redirects to when clicked on."
    );

    /// The timestamp payload object for the activity, containing the start and optionally the end timestamps.
    #[must_use]
    pub fn timestamps(&self) -> &TimestampPayload {
        &self.timestamps
    }
    /// The assets payload object for the activity, containing all the detials of the
    /// assets which were optionally sent with the activity.
    #[must_use]
    pub fn assets(&self) -> Option<&AssetsPayload> {
        self.assets.as_ref()
    }
    /// The buttons for the activity. Each button is represented by a [`ButtonPayload`] object instance, and returned as a borrowed
    /// vector of multiple [`ButtonPayload`] objects through this function.
    #[must_use]
    pub fn buttons(&self) -> Option<&Vec<ButtonPayload>> {
        self.buttons.as_ref()
    }

    pub(crate) fn create(
        value: ActivitySpec,
        session_start: u64,
    ) -> Result<Self, InnerParsingError> {
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
