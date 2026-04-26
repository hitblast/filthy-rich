use anyhow::anyhow;
use tokio::sync::{mpsc::Sender, oneshot};

use crate::types::{Activity, IPCCommand};

/// A client handle for communicating with [`super::PresenceRunner`] and its inner loop.
#[derive(Debug, Clone)]
pub struct PresenceClient {
    pub(crate) tx: Sender<IPCCommand>,
    pub(crate) client_id: String,
}

impl PresenceClient {
    /// Returns the client ID.
    #[must_use]
    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    /// Sets/updates the Discord Rich presence activity.
    /// The runner must be started before calling this.
    pub async fn set_activity(&self, activity: Activity) -> Result<(), anyhow::Error> {
        self.tx
            .send(IPCCommand::SetActivity {
                activity: Box::new(activity),
            })
            .await
            .map_err(|_| anyhow!("Connection has already been closed."))?;

        Ok(())
    }

    /// Clears a previously set Discord Rich Presence activity.
    pub async fn clear_activity(&self) -> Result<(), anyhow::Error> {
        self.tx
            .send(IPCCommand::ClearActivity)
            .await
            .map_err(|_| anyhow!("Connection has already been closed."))?;

        Ok(())
    }

    /// Closes the current connection if any.
    pub async fn close(&self) -> Result<(), anyhow::Error> {
        let (done_tx, done_rx) = oneshot::channel::<()>();

        if self
            .tx
            .send(IPCCommand::Close { done: done_tx })
            .await
            .is_err()
        {
            return Ok(());
        }

        done_rx.await?;

        Ok(())
    }
}
