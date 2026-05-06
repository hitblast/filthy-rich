use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::{
    errors::InnerParsingError,
    types::{ActivitySpec, ActivityType, StatusDisplayType},
    utils::get_current_timestamp,
};

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
    pub fn large_image(&self) -> Option<&str> {
        self.large_image.as_deref()
    }
    pub fn large_url(&self) -> Option<&str> {
        self.large_url.as_deref()
    }
    pub fn large_text(&self) -> Option<&str> {
        self.large_text.as_deref()
    }
    pub fn small_image(&self) -> Option<&str> {
        self.small_image.as_deref()
    }
    pub fn small_url(&self) -> Option<&str> {
        self.small_url.as_deref()
    }
    pub fn small_text(&self) -> Option<&str> {
        self.small_text.as_deref()
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonPayload {
    pub(crate) label: String,
    pub(crate) url: String,
}

impl ButtonPayload {
    pub fn label(&self) -> &str {
        self.label.as_ref()
    }
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimestampPayload {
    start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<u64>,
}

impl TimestampPayload {
    pub fn start(&self) -> u64 {
        self.start
    }
    pub fn end(&self) -> Option<u64> {
        self.end
    }
}

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
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn activity_type(&self) -> Option<ActivityType> {
        self.r#type
    }
    pub fn created_at(&self) -> u64 {
        self.created_at
    }
    pub fn instance(&self) -> Option<bool> {
        self.instance
    }
    pub fn status_display_type(&self) -> Option<StatusDisplayType> {
        self.status_display_type
    }
    pub fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }
    pub fn details_url(&self) -> Option<&str> {
        self.details_url.as_deref()
    }
    pub fn state(&self) -> Option<&str> {
        self.state.as_deref()
    }
    pub fn state_url(&self) -> Option<&str> {
        self.state_url.as_deref()
    }
    pub fn timestamps(&self) -> &TimestampPayload {
        &self.timestamps
    }
    pub fn assets(&self) -> Option<&AssetsPayload> {
        self.assets.as_ref()
    }
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
