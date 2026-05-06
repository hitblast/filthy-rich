use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    errors::InnerParsingError,
    types::{ActivityPayload, ActivitySpec, ReadyData},
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

#[derive(Deserialize)]
pub(crate) struct ReadyRPCFrame {
    pub cmd: Option<String>,
    pub evt: Option<String>,
    pub data: Option<ReadyData>,
}

#[derive(Debug, Deserialize)]
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
        activity: Box<ActivitySpec>,
    },
    ClearActivity,
    Close {
        done_tx: tokio::sync::oneshot::Sender<()>,
    },
}
