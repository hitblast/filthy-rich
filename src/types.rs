use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct IPCActivityCmd {
    cmd: String,
    args: IPCActivityCmdArgs,
    nonce: String,
}
impl IPCActivityCmd {
    pub fn new_with(activity: Option<ActivityPayload>) -> Self {
        Self {
            cmd: "SET_ACTIVITY".to_string(),
            args: IPCActivityCmdArgs {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct IPCActivityCmdArgs {
    pid: u32,
    activity: Option<ActivityPayload>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityPayload {
    pub(crate) details: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) state: Option<String>,
    pub(crate) timestamps: TimestampPayload,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimestampPayload {
    pub(crate) start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) end: Option<u64>,
}
