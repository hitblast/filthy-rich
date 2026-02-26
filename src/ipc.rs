// SPDX-License-Identifier: MIT

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, bail};
use serde_json::json;
use tokio::{
    runtime::{Builder, Runtime},
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    task::JoinHandle,
    time::sleep,
};

use crate::{
    socket::DiscordIPCSocket,
    types::{Activity, ActivityPayload, IPCActivityCmd, IPCCommand, ReadyData, RpcFrame, TimestampPayload},
};

fn get_current_timestamp() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs())
}

/// Primary struct for you to set and update Discord Rich Presences with.
pub struct DiscordIPC {
    tx: Sender<IPCCommand>,
    client_id: String,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
    on_ready: Option<Box<dyn Fn(ReadyData) + Send + Sync + 'static>>,
}

impl DiscordIPC {
    pub fn new(client_id: &str) -> Self {
        let (tx, _rx) = mpsc::channel(32);

        Self {
            tx,
            client_id: client_id.to_string(),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            on_ready: None,
        }
    }

    /// Run a particular closure after receiving the READY event from the local Discord IPC server.
    pub fn on_ready<F: Fn(ReadyData) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
        self
    }

    /// Returns the client ID.
    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    /// Run the client.
    /// Must be called before any [`DiscordIPC::set_activity`] calls.
    pub async fn run(&mut self, wait_for_ready: bool) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            bail!(
                "Cannot run multiple instances of .run() for DiscordIPC, or when a session is still closing."
            )
        }

        let (tx, mut rx) = mpsc::channel::<IPCCommand>(32);
        self.tx = tx;
        let client_id = self.client_id.clone();
        let running = self.running.clone();

        // oneshot channel to signal when READY is received the first time
        let (ready_tx, ready_rx) = oneshot::channel::<()>();

        let on_ready = self.on_ready.take();

        let handle = tokio::spawn(async move {
            let mut backoff = 1;
            let mut last_activity: Option<Activity> = None;
            let mut ready_tx = Some(ready_tx);

            while running.load(Ordering::SeqCst) {
                // initial connect
                let mut socket = match DiscordIPCSocket::new().await {
                    Ok(s) => s,
                    Err(_) => {
                        sleep(Duration::from_secs(backoff)).await;
                        continue;
                    }
                };

                // initial handshake
                if socket.do_handshake(&client_id).await.is_err() {
                    sleep(Duration::from_secs(backoff)).await;
                    continue;
                }

                // wait for READY (blocks until first READY or socket fails)
                loop {
                    let frame = match socket.read_frame().await {
                        Ok(f) => f,
                        Err(_) => break,
                    };

                    if frame.opcode != 1 {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_slice::<RpcFrame>(&frame.body) {
                        if json.cmd.as_deref() == Some("DISPATCH")
                            && json.evt.as_deref() == Some("READY")
                        {
                            if let Some(tx) = ready_tx.take() {
                                let _ = tx.send(());
                            }
                            if let Some(f) = on_ready.as_ref() {
                                if let Some(data) = json.data {
                                    f(data);
                                }
                            }
                            break;
                        }

                        if json.evt.as_deref() == Some("ERROR") {
                            eprintln!("Discord RPC error: {:?}", json.data);
                        }
                    }
                }

                let session_start = get_current_timestamp()?;

                if let Some(activity) = &last_activity {
                    let _ = socket.send_activity(activity.clone(), session_start).await;
                }

                backoff = 1;

                loop {
                    tokio::select! {
                        Some(cmd) = rx.recv() => {
                            match cmd {
                                IPCCommand::SetActivity { activity } => {
                                    last_activity = Some(activity.clone());

                                    if socket.send_activity(activity, session_start).await.is_err() {
                                        break;
                                    }
                                },
                                IPCCommand::ClearActivity => {
                                    last_activity = None;

                                    if socket.clear_activity().await.is_err() { break; }
                                },
                                IPCCommand::Close => {
                                    let _ = socket.close().await;
                                    running.store(false, Ordering::SeqCst);
                                    break;
                                }
                            }
                        }

                        frame = socket.read_frame() => {
                            match frame {
                                Ok(frame) => match frame.opcode {
                                    1 => {
                                        if let Ok(json) = serde_json::from_slice::<RpcFrame>(&frame.body) {
                                            if json.evt.as_deref() == Some("ERROR") {
                                                eprintln!("Discord RPC error: {:?}", json.data);
                                            }
                                        }
                                    },
                                    2 => break,
                                    3 => {
                                        if socket.send_frame(3, frame.body).await.is_err() { break; }
                                    },
                                    _ => {}
                                },
                                Err(_) => break,
                            }
                        }
                    }
                }

                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(4);
            }

            Ok(())
        });

        self.handle = Some(handle);

        if wait_for_ready {
            match ready_rx.await {
                Ok(()) => Ok(()),
                Err(_) => bail!("Background task exited before READY."),
            }
        } else {
            Ok(())
        }
    }

    /// Waits for the IPC task to finish.
    pub async fn wait(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await??;
        }

        Ok(())
    }

    /// Checks if the task is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Sets/updates the Discord Rich presence activity.
    /// [`DiscordIPC::run`] must be executed prior to calling this.
    pub async fn set_activity(&self, activity: Activity) -> Result<()> {
        if !self.is_running() {
            bail!("Call .run() before .set_activity() execution.");
        }

        self.tx.send(IPCCommand::SetActivity { activity }).await?;
        Ok(())
    }

    /// Clears a previously set Discord Rich Presence activity.
    pub async fn clear_activity(&self) -> Result<()> {
        if self.is_running() {
            self.tx.send(IPCCommand::ClearActivity).await?;
        }

        Ok(())
    }

    /// Closes the current connection if any.
    pub async fn close(&mut self) -> Result<()> {
        if self.is_running() {
            let _ = self.tx.send(IPCCommand::Close).await;
            if let Some(handle) = self.handle.take() {
                if let Err(e) = handle.await {
                    eprintln!("DiscordIPC background task failed on close: {e:?}");
                }
            }
        }

        Ok(())
    }
}

impl DiscordIPCSocket {
    pub(crate) async fn send_activity(&mut self, activity: Activity, session_start: u64) -> Result<()> {
        let current_t = get_current_timestamp()?;
        let end_timestamp = activity.duration.map(|d| current_t + d.as_secs());

        let assets = if activity.large_image_key.is_some() || activity.small_image_key.is_some() {
            Some(crate::types::AssetsPayload {
                large_image: activity.large_image_key,
                large_text: activity.large_image_text,
                small_image: activity.small_image_key,
                small_text: activity.small_image_text,
            })
        } else {
            None
        };

        let cmd = IPCActivityCmd::new_with(Some(ActivityPayload {
            details: activity.details,
            state: activity.state,
            timestamps: TimestampPayload {
                start: session_start,
                end: end_timestamp,
            },
            assets,
        }));

        self.send_cmd(cmd).await?;
        Ok(())
    }

    pub(crate) async fn clear_activity(&mut self) -> Result<()> {
        let cmd = IPCActivityCmd::new_with(None);
        self.send_cmd(cmd).await?;
        Ok(())
    }

    pub(crate) async fn do_handshake(&mut self, client_id: &str) -> Result<()> {
        let handshake = json!({ "v": 1, "client_id": client_id }).to_string();
        self.send_frame(0, handshake).await?;
        Ok(())
    }

    async fn send_cmd(&mut self, cmd: IPCActivityCmd) -> Result<()> {
        let cmd = cmd.to_json()?;
        self.send_frame(1, cmd).await?;
        Ok(())
    }
}

/// Blocking implementation of [`DiscordIPC`].
pub struct DiscordIPCSync {
    inner: DiscordIPC,
    rt: Runtime,
}

impl DiscordIPCSync {
    pub fn new(client_id: &str) -> Result<Self> {
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let inner = DiscordIPC::new(client_id);

        Ok(Self { inner, rt })
    }

    /// Run a particular closure after receiving the READY event from the local Discord IPC server.
    pub fn on_ready<F: Fn(ReadyData) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.inner.on_ready = Some(Box::new(f));
        self
    }

    /// Returns the client ID.
    pub fn client_id(&self) -> String {
        self.inner.client_id()
    }

    /// Run the client.
    /// Must be called before any [`DiscordIPCSync::set_activity`] calls.
    pub fn run(&mut self, wait_for_ready: bool) -> Result<()> {
        self.rt.block_on(self.inner.run(wait_for_ready))
    }

    /// Waits for the IPC task to finish.
    /// [`DiscordIPCSync::run`] must be called to spawn it in the first place.
    pub fn wait(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.wait())
    }

    /// Checks if the task is running.
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Sets/updates the Discord Rich presence activity.
    /// [`DiscordIPCSync::run`] must be executed prior to calling this.
    pub fn set_activity(&self, activity: Activity) -> Result<()> {
        self.rt.block_on(self.inner.set_activity(activity))
    }

    /// Clears a previously set Discord Rich Presence activity.
    pub fn clear_activity(&self) -> Result<()> {
        self.rt.block_on(self.inner.clear_activity())
    }

    /// Closes the current connection if any.
    pub fn close(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.close())
    }
}

impl Drop for DiscordIPCSync {
    fn drop(&mut self) {
        let _ = self.rt.block_on(self.inner.close());
    }
}
