use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{mpsc::Sender, oneshot};

use crate::types::{Activity, IPCCommand};

/// A client handle for communicating with the [`PresenceRunner`]'s inner loop.
#[derive(Debug, Clone)]
pub struct PresenceClient {
    pub(crate) tx: Sender<IPCCommand>,
    pub(crate) client_id: String,
    pub(crate) running: Arc<AtomicBool>,
}

impl PresenceClient {
    /// Returns the client ID.
    #[must_use]
    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    /// Checks if the task is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Sets/updates the Discord Rich presence activity.
    /// The runner must be started before calling this.
    pub async fn set_activity(&self, activity: Activity) -> Result<(), anyhow::Error> {
        if !self.is_running() {
            anyhow::bail!("Call .run() before .set_activity() execution.");
        }

        self.tx
            .send(IPCCommand::SetActivity {
                activity: Box::new(activity),
            })
            .await?;
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
            let (done_tx, done_rx) = oneshot::channel::<()>();
            self.tx.send(IPCCommand::Close { done: done_tx }).await?;
            done_rx.await?;
        }

        Ok(())
    }
}
