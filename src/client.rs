use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    errors::PresenceClientError,
    types::{Activity, IPCCommand},
};

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
    pub async fn set_activity(&self, activity: Activity) -> Result<(), PresenceClientError> {
        let activity = Box::new(activity);

        self.tx
            .send(IPCCommand::SetActivity { activity })
            .await
            .map_err(|_| PresenceClientError::ActivitySendError)?;

        Ok(())
    }

    /// Clears a previously set Discord Rich Presence activity.
    pub async fn clear_activity(&self) -> Result<(), PresenceClientError> {
        self.tx
            .send(IPCCommand::ClearActivity)
            .await
            .map_err(|_| PresenceClientError::ActivitySendError)?;

        Ok(())
    }

    /// Closes the current connection if any.
    pub async fn close(&self) -> Result<(), PresenceClientError> {
        let (done_tx, done_rx) = oneshot::channel::<()>();

        match self.tx.send(IPCCommand::Close { done_tx }).await {
            Ok(_) => done_rx.await?,
            Err(_) => return Ok(()),
        }

        Ok(())
    }
}
