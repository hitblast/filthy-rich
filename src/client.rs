use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::Sender;

use crate::types::IPCCommand;
use crate::Activity;

/// A client handle for controlling Discord IPC.
#[derive(Clone)]
pub struct DiscordIPCClient {
    pub(crate) tx: Sender<IPCCommand>,
    pub(crate) client_id: String,
    pub(crate) running: Arc<AtomicBool>,
}

impl DiscordIPCClient {
    /// Returns the client ID.
    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    /// Checks if the task is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Sets/updates the Discord Rich presence activity.
    /// The runner must be started before calling this.
    pub async fn set_activity(&self, activity: Activity) -> Result<(), anyhow::Error> {
        if !self.is_running() {
            anyhow::bail!("Call .run() before .set_activity() execution.");
        }

        self.tx.send(IPCCommand::SetActivity { activity }).await?;
        Ok(())
    }

    /// Clears a previously set Discord Rich Presence activity.
    pub async fn clear_activity(&self) -> Result<(), anyhow::Error> {
        if self.is_running() {
            self.tx.send(IPCCommand::ClearActivity).await?;
        }

        Ok(())
    }

    /// Closes the current connection if any.
    pub async fn close(&self) -> Result<(), anyhow::Error> {
        if self.is_running() {
            let _ = self.tx.send(IPCCommand::Close).await;
        }

        Ok(())
    }
}