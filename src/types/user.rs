use serde::Deserialize;
use serde_json::Value;

use crate::ds;

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

    ds!(global_name, "The global name for the user.");
    ds!(discriminator, "The discriminator of the user.");
    ds!(avatar, "The avatar of the user.");

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
