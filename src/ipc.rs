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
    activitytypes::{ActivityPayload, IPCActivityCmd, TimestampPayload},
    socket::DiscordIPCSocket,
};

/*
 *
 * Helper funcs
 *
 */

fn get_current_timestamp() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

fn pack(opcode: u32, data_len: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&opcode.to_le_bytes());
    bytes.extend_from_slice(&data_len.to_le_bytes());
    bytes
}

/*
 *
 * Frame/cmd structs
 *
 */

#[derive(Debug)]
enum IPCCommand {
    SetActivity {
        details: String,
        state: Option<String>,
    },
    ClearActivity,
    Close,
}

#[derive(Debug, Deserialize)]
struct RpcFrame {
    cmd: Option<String>,
    evt: Option<String>,
    data: Option<serde_json::Value>,
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
    on_ready: Option<Box<dyn Fn() + Send + 'static>>,
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

    /// Run anything on the READY event of the Discord IPC task instance.
    pub fn on_ready<F: Fn() + Send + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
        self
    }

    /// The Discord client ID that has been used to initialize this IPC client instance.
    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    /// Run the client.
    /// Must be called before any [`set_activity()`] calls.
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
            let mut last_activity: Option<(String, Option<String>)> = None;
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
                let handshake = json!({ "v": 1, "client_id": client_id }).to_string();
                let packed = pack(0, handshake.len() as u32);

                if socket.write(&packed).await.is_err()
                    || socket.write(handshake.as_bytes()).await.is_err()
                {
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
                                let _ = tx.send(()); // moves the sender here exactly once
                            }
                            if let Some(f) = &on_ready {
                                f();
                            }
                            break;
                        }
                        if json.evt.as_deref() == Some("ERROR") {
                            eprintln!("Discord RPC error: {:?}", json.data);
                        }
                    }
                }

                let timestamp = get_current_timestamp()?;

                // reset activity if previous instance failed and this instance is basically reconnecting
                if let Some((details, state)) = &last_activity {
                    let _ =
                        send_activity(&mut socket, details.clone(), state.clone(), timestamp).await;
                }

                backoff = 1;

                loop {
                    tokio::select! {
                        Some(cmd) = rx.recv() => {
                            match cmd {
                                IPCCommand::SetActivity { details, state } => {
                                    last_activity = Some((details.clone(), state.clone()));
                                    if send_activity(&mut socket, details, state, timestamp).await.is_err() {
                                        break;
                                    }
                                },
                                IPCCommand::ClearActivity => {
                                    last_activity = None;
                                    if clear_activity(&mut socket).await.is_err() { break; }
                                },
                                IPCCommand::Close => {
                                    let json = b"{}";
                                    let packed = pack(2, json.len() as u32);
                                    let _ = socket.write(&packed).await;
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
                                        let packed = pack(3, frame.body.len() as u32);
                                        if socket.write(&packed).await.is_err() { break; }
                                        if socket.write(&frame.body).await.is_err() { break; }
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

    /// Sets/updates the Discord Rich presence activity.
    /// [`run()`] must be executed prior to calling this.
    pub async fn set_activity(&self, details: String, state: Option<String>) -> Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            bail!("Call .run() before .set_activity() execution.");
        }

        self.tx
            .send(IPCCommand::SetActivity { details, state })
            .await?;
        Ok(())
    }

    /// Clears a previously set Discord Rich Presence activity.
    pub async fn clear_activity(&self) -> Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.tx.send(IPCCommand::ClearActivity).await?;
        Ok(())
    }

    /// Closes the current session of Rich Presence activity.
    pub async fn close(&mut self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
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

async fn send_activity(
    socket: &mut DiscordIPCSocket,
    details: String,
    state: Option<String>,
    timestamp: u64,
) -> Result<()> {
    let cmd = IPCActivityCmd::new_with(Some(ActivityPayload {
        details,
        state,
        timestamps: TimestampPayload { start: timestamp },
    }))
    .to_json()?;

    let packed = pack(1, cmd.len() as u32);
    socket.write(&packed).await?;
    socket.write(cmd.as_bytes()).await?;
    Ok(())
}

async fn clear_activity(socket: &mut DiscordIPCSocket) -> Result<()> {
    let cmd = IPCActivityCmd::new_with(None).to_json()?;

    let packed = pack(1, cmd.len() as u32);
    socket.write(&packed).await?;
    socket.write(cmd.as_bytes()).await?;
    Ok(())
}

/*
 *
 * Blocking implementation
 *
 */

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

    /// Run anything on the READY event of the Discord IPC task instance.
    pub fn on_ready<F: Fn() + Send + 'static>(mut self, f: F) -> Self {
        self.inner.on_ready = Some(Box::new(f));
        self
    }

    /// The Discord client ID that has been used to initialize this IPC client instance.
    pub fn client_id(&self) -> String {
        self.inner.client_id()
    }

    /// Run the client.
    /// Must be called before any [`set_activity()`] calls.
    pub fn run(&mut self, wait_for_ready: bool) -> Result<()> {
        self.rt.block_on(self.inner.run(wait_for_ready))
    }

    /// Waits for the IPC task to finish.
    /// [`run()`] must be called to spawn it in the first place.
    pub fn wait(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.wait())
    }

    /// Sets/updates the Discord Rich presence activity.
    /// [`run()`] must be executed prior to calling this.
    pub fn set_activity(&self, details: String, state: Option<String>) -> Result<()> {
        self.rt.block_on(self.inner.set_activity(details, state))
    }

    /// Clears a previously set Discord Rich Presence activity.
    pub fn clear_activity(&self) -> Result<()> {
        self.rt.block_on(self.inner.clear_activity())
    }

    /// Closes the current session of Rich Presence activity.
    pub fn close(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.close())
    }
}

impl Drop for DiscordIPCSync {
    fn drop(&mut self) {
        let _ = self.rt.block_on(self.inner.close());
    }
}
