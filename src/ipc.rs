// SPDX-License-Identifier: MIT

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, bail};
use serde::Deserialize;
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
    types::{ActivityPayload, IPCActivityCmd, TimestampPayload},
};

/*
 *
 * Helper funcs
 *
 */

fn get_current_timestamp() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/*
 *
 * Frame/cmd structs
 *
 */

#[derive(Debug)]
enum IPCCommand {
    SetActivity { activity: Activity },
    ClearActivity,
    Close,
}

#[derive(Debug, Deserialize)]
struct RpcFrame {
    cmd: Option<String>,
    evt: Option<String>,
    data: Option<ReadyData>,
}

/// An object which is passed during READY capture from Discord IPC instance.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadyData {
    pub user: DiscordUser,
}

/// Represents a Discord user.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    pub avatar_decoration_data: Option<serde_json::Value>,
    pub bot: bool,
    pub flags: Option<u64>,
    pub premium_type: Option<u64>,
}

/// Represents a Discord Rich Presence activity.
#[derive(Debug, Clone)]
pub struct Activity {
    details: Option<String>,
    state: Option<String>,
    duration: Option<Duration>,
}

pub struct ActivityBuilder;

/// A Rich Presence activity with top text and possibly more attributes.
/// [`ActivityWithDetails::build`] needs to be called on it in order to
/// turn it into a proper [`Activity`] instance.
pub struct ActivityWithDetails {
    details: String,
    state: Option<String>,
    duration: Option<Duration>,
}

impl Activity {
    pub fn new() -> ActivityBuilder {
        ActivityBuilder
    }

    /// Initializes a Rich Presence activity without any content; useful for small apps.
    pub fn build_empty() -> Self {
        Self {
            details: None,
            state: None,
            duration: None,
        }
    }
}

impl ActivityBuilder {
    /// Top text for your activity.
    pub fn details(self, details: impl Into<String>) -> ActivityWithDetails {
        ActivityWithDetails {
            details: details.into(),
            state: None,
            duration: None,
        }
    }
}

impl ActivityWithDetails {
    /// Bottom text for your activity.
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Countdown duration for your activity.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Parses the state of this builder into a usable [`Activity`] for you to pass through either [`DiscordIPC::set_activity`]
    /// or [`DiscordIPCSync::set_activity`].
    pub fn build(self) -> Activity {
        Activity {
            details: Some(self.details),
            state: self.state,
            duration: self.duration,
        }
    }
}

/*
 *
 * Async implementation
 *
 */

/// Primary struct for you to set and update Discord Rich Presences with.
pub struct DiscordIPC {
    tx: Sender<IPCCommand>,
    client_id: String,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
    on_ready: Option<Box<dyn Fn(ReadyData) + Send + Sync + 'static>>,
}

impl DiscordIPC {
    /// Creates a new Discord IPC client instance.
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

    /// The Discord client ID that has been used to initialize this IPC client instance.
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

            'outer: while running.load(Ordering::SeqCst) {
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
                        Err(_) => continue 'outer,
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
                            if let (Some(f), Some(data)) = (&on_ready, json.data) {
                                f(data);
                            }
                            break;
                        }

                        if json.evt.as_deref() == Some("ERROR") {
                            eprintln!("Discord RPC error: {:?}", json.data);
                        }
                    }
                }

                // should reset per new run() call
                let session_start = get_current_timestamp()?;

                // reset activity if previous instance failed and this instance is basically reconnecting
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
                                    break 'outer;
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
                                    }
                                    2 => break,
                                    3 => {
                                        if socket.send_frame(3, frame.body).await.is_err() { break; }
                                    }
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

    /// Checks whether the task is running through the internal atomic indicator flag.
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

/*
 *
 * Extension of DiscordIPCSocket
 * (for convenience)
 *
 */

impl DiscordIPCSocket {
    async fn send_activity(&mut self, activity: Activity, session_start: u64) -> Result<()> {
        let current_t = get_current_timestamp()?;
        let end_timestamp = activity.duration.map(|d| current_t + d.as_secs());

        let cmd = IPCActivityCmd::new_with(Some(ActivityPayload {
            details: activity.details,
            state: activity.state,
            timestamps: TimestampPayload {
                start: session_start,
                end: end_timestamp,
            },
        }));

        self.send_cmd(cmd).await
    }

    async fn clear_activity(&mut self) -> Result<()> {
        let cmd = IPCActivityCmd::new_with(None);
        self.send_cmd(cmd).await?;
        Ok(())
    }

    async fn do_handshake(&mut self, client_id: &str) -> Result<()> {
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

/*
 *
 * Blocking implementation
 *
 */

/// Blocking implementation of [`DiscordIPC`].
pub struct DiscordIPCSync {
    inner: DiscordIPC,
    rt: Runtime,
}

impl DiscordIPCSync {
    /// Creates a new Discord IPC client instance.
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

    /// The Discord client ID that has been used to initialize this IPC client instance.
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

    /// Checks whether the task is running through the internal atomic indicator flag.
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
