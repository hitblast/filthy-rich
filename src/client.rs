use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    errors::PresenceClientError,
    str,
    types::{ActivitySpec, IPCCommand},
};

/// A client handle for communicating with [`super::PresenceRunner`] and its inner loop.
#[derive(Debug, Clone)]
pub struct PresenceClient {
    pub(crate) tx: Sender<IPCCommand>,
    pub(crate) client_id: String,
}

impl PresenceClient {
    str!(client_id, "Returns the client ID.");

    /// Sets/updates the Discord Rich presence activity.
    /// The runner must be started before calling this.
    ///
    /// NOTE: This will NOT wait for the activity to finish becoming online.
    pub async fn set_activity(&self, activity: ActivitySpec) -> Result<(), PresenceClientError> {
        let activity = Box::new(activity);

        self.tx
            .send(IPCCommand::SetActivity { activity })
            .await
            .map_err(|_| PresenceClientError::ActivitySendError)?;

        Ok(())
    }

    /// Clears a previously set Discord Rich Presence activity.
    ///
    /// NOTE: This will NOT wait for the activity to finish clearing.
    pub async fn clear_activity(&self) -> Result<(), PresenceClientError> {
        self.tx
            .send(IPCCommand::ClearActivity)
            .await
            .map_err(|_| PresenceClientError::ActivitySendError)?;

        Ok(())
    }

    /// Closes the current connection if any.
    ///
    /// This is a semi-blocking call and does wait for the runner thread to respond to the signal being sent.
    pub async fn close(&self) -> Result<(), PresenceClientError> {
        let (done_tx, done_rx) = oneshot::channel::<()>();

        match self.tx.send(IPCCommand::Close { done_tx }).await {
            Ok(_) => done_rx.await?,
            Err(_) => return Ok(()),
        }

        Ok(())
    }
}
