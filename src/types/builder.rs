use std::{collections::HashMap, time::Duration};

use crate::{
    types::{ActivitySpec, ActivityType, AssetsPayload, ButtonPayload, StatusDisplayType},
    utils::filter_none_string,
};

/// Represents a Discord Rich Presence activity which is yet to be built. To start building it into a usable [`ActivitySpec`],
/// initialize a new [`ActivityBuilder`] with [`Activity::new`].
pub struct Activity;

impl Activity {
    /// Initializes a new activity builder instance.
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ActivityBuilder {
        ActivityBuilder::default()
    }

    /// Gives out an empty but usable [`ActivitySpec`]. Essentially,
    /// this only shows the name of the app and the elapsed time for the activity on
    /// Discord. Useful when you only need a simple rich presence instance.
    ///
    /// For building a complete activity, using [`Activity::new`] is suggested instead.
    ///
    /// NOTE: This is the same as calling [`ActivitySpec::default`].
    #[must_use]
    pub fn empty_spec() -> ActivitySpec {
        ActivitySpec::default()
    }
}

/// A builder for a Rich Presence activity.
/// To build a [`ActivitySpec`] out of it, use [`ActivityBuilder::build`].
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

    /// Parses the state of this builder into a usable [`ActivitySpec`] for you to pass through [`super::PresenceClient::set_activity`].
    #[must_use]
    pub fn build(self) -> ActivitySpec {
        ActivitySpec {
            name: self.name,
            r#type: self.activity_type,
            status_display_type: self.status_display_type,
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
