use serde::Deserialize;
use serde_json::Value;

use crate::{ds, types::ActivityPayload};

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
    ds!(application_id, "The ID of the application");
    ds!(platform, "The platform of the host.");

    /// Undocumented metadata.
    #[must_use]
    pub fn metadata(&self) -> Option<&Value> {
        self.metadata.as_ref()
    }

    /// The activity payload which came in response to the send.
    #[must_use]
    pub fn activity(&self) -> &ActivityPayload {
        &self.activity
    }
}

/// Data received from a READY event.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadyData {
    v: u8,
    config: ServerConfigurationData,
    user: DiscordUser,
}

impl ReadyData {
    /// The user to whom you are connected.
    #[must_use]
    pub fn user(&self) -> &DiscordUser {
        &self.user
    }

    /// The version of the RPC that is being used.
    #[must_use]
    pub fn version(&self) -> u8 {
        self.v
    }

    /// The server configuration data for the RPC.
    #[must_use]
    pub fn config(&self) -> &ServerConfigurationData {
        &self.config
    }
}

/// Server configuration data received from the RPC server.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfigurationData {
    cdn_host: String,
    api_endpoint: String,
    environment: String,
}

impl ServerConfigurationData {
    /// The CDN for the RPC server.
    #[must_use]
    pub fn cdn_host(&self) -> &str {
        &self.cdn_host
    }

    /// The API endpoint for the RPC server.
    #[must_use]
    pub fn api_endpoint(&self) -> &str {
        &self.api_endpoint
    }

    /// The environment for the RPC server.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }
}

/// Represents the Discord user that the RPC connection is present with.
///
/// NOTE: Only the fields which may need a documentation have been given one.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
    global_name: Option<String>,
    avatar: Option<String>,
    bot: Option<bool>,
    system: Option<bool>,
    mfa_enabled: Option<bool>,
    banner: Option<String>,
    accent_color: Option<isize>,
    locale: Option<String>,
    verified: Option<bool>,
    email: Option<String>,
    flags: Option<isize>,
    premium_type: Option<isize>,
    public_flags: Option<isize>,
    avatar_decoration_data: Option<Value>,
    collectibles: Option<Value>,
    primary_guild: Option<Value>,
}

impl DiscordUser {
    /// The user's ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The user's username, not unique across the platform.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// The user's Discord-tag.
    #[must_use]
    pub fn discriminator(&self) -> &str {
        &self.discriminator
    }

    ds!(global_name, "The user's display name, if set.");
    ds!(avatar, "The user's avatar hash.");

    /// Whether the user belongs to an OAuth2 application.
    #[must_use]
    pub fn bot(&self) -> Option<bool> {
        self.bot
    }

    /// Whether the user is an Official Discord System user (part of the urgent message system).
    #[must_use]
    pub fn system(&self) -> Option<bool> {
        self.system
    }

    /// Whether the user has two factor enabled on their account.
    #[must_use]
    pub fn mfa_enabled(&self) -> Option<bool> {
        self.mfa_enabled
    }

    ds!(banner, "The user's banner hash.");
    ds!(locale, "The user's chosen language option.");
    ds!(email, "The user's email.");

    /// Whether the email on this account has been verified.
    #[must_use]
    pub fn verified(&self) -> Option<bool> {
        self.verified
    }

    /// The user's banner color encoded as an integer representation of hexadecimal color code.
    #[must_use]
    pub fn accent_color(&self) -> Option<isize> {
        self.accent_color
    }

    /// The flags on the user's account.
    #[must_use]
    pub fn flags(&self) -> Option<isize> {
        self.flags
    }

    /// The type of Nitro subscription on the user's account.
    #[must_use]
    pub fn premium_type(&self) -> Option<isize> {
        self.premium_type
    }

    /// The public flags on the user's account.
    #[must_use]
    pub fn public_flags(&self) -> Option<isize> {
        self.public_flags
    }

    /// Data for the user's avatar decoration.
    #[must_use]
    pub fn avatar_decoration_data(&self) -> Option<&Value> {
        self.avatar_decoration_data.as_ref()
    }

    /// Data for the user's collectibles.
    #[must_use]
    pub fn collectibles(&self) -> Option<&Value> {
        self.collectibles.as_ref()
    }

    /// The user's primary guild.
    #[must_use]
    pub fn primary_guild(&self) -> Option<&Value> {
        self.primary_guild.as_ref()
    }
}
