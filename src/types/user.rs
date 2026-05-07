use serde::Deserialize;
use serde_json::Value;

use crate::ds;

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
