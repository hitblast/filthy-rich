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
    sync::mpsc::{self, Sender},
    task::JoinHandle,
    time::{sleep, timeout},
};
use uuid::Uuid;

use crate::socket::DiscordIPCSocket;

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
 * General functionality
 *
 */

#[derive(Debug)]
enum IPCCommand {
    SetActivity { details: String, state: String },
    ClearActivity,
    Close,
}

#[derive(Debug, Deserialize)]
struct RpcFrame {
    cmd: Option<String>,
    evt: Option<String>,
    data: Option<serde_json::Value>,
}

/// Primary struct for you to set and update Discord Rich Presences with.
#[derive(Debug)]
pub struct DiscordIPC {
    tx: Sender<IPCCommand>,
    client_id: String,
    handle: Option<JoinHandle<Result<()>>>,
    running: Arc<AtomicBool>,
}

impl DiscordIPC {
    /// Creates a new asynchronous Discord IPC client with the given client ID.
    #[must_use]
    pub fn new(client_id: &str) -> Self {
        let (tx, _rx) = mpsc::channel(32);

        Self {
            tx,
            client_id: client_id.to_string(),
            handle: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the Discord client ID that has been used to initialize this IPC client.
    #[must_use]
    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    /// Checks internally whether or not an existing IPC client instance
    /// is running and currently attached with Discord.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst) && self.handle.is_some()
    }

    /// Runs the Discord IPC client.
    pub async fn run(&mut self) -> Result<()> {
        if self.handle.is_some() && self.running.swap(true, Ordering::SeqCst) {
            bail!("Cannot run multiple instances of .run().");
        }

        let running = self.running.clone();

        let (tx, mut rx) = mpsc::channel::<IPCCommand>(32);
        self.tx = tx;
        let client_id = self.client_id.clone();

        let handle = tokio::spawn(async move {
            let mut backoff = 1;
            let mut last_activity: Option<(String, String)> = None;
            let timestamp = get_current_timestamp()?;

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
                let handshake = json!({ "v": 1, "client_id": client_id }).to_string();
                let packed = pack(0, handshake.len() as u32);

                if socket.write(&packed).await.is_err()
                    || socket.write(handshake.as_bytes()).await.is_err()
                {
                    sleep(Duration::from_secs(backoff)).await;
                    continue;
                }

                // wait for ready
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
                            break;
                        }
                        if json.evt.as_deref() == Some("ERROR") {
                            eprintln!("Discord RPC error: {:?}", json.data);
                        }
                    }
                }

                // reset activity if previous instance failed and this instance is basically reconnecting
                if let Some((details, state)) = &last_activity {
                    let _ = send_activity(&mut socket, details, state, timestamp).await;
                }

                backoff = 1;

                loop {
                    tokio::select! {
                        Some(cmd) = rx.recv() => {
                            match cmd {
                                IPCCommand::SetActivity { details, state } => {
                                    last_activity = Some((details.clone(), state.clone()));

                                    if send_activity(&mut socket, &details, &state, timestamp).await.is_err() {
                                        break;
                                    }
                                },
                                IPCCommand::ClearActivity => {
                                    if clear_activity(&mut socket).await.is_err() {
                                        break;
                                    }
                                }
                                IPCCommand::Close => {
                                    clear_activity(&mut socket).await?;
                                    socket.close().await?;
                                    running.store(false, Ordering::SeqCst);
                                    return Ok(());
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

        Ok(())
    }

    /// Waits for the primary IPC client task to finish.
    /// Can also be used to keep the IPC client running forever.
    pub async fn wait(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await??;
        }
        Ok(())
    }

    /// Sets/updates a Discord Rich Presence activity.
    pub async fn set_activity(&self, details: &str, state: &str) -> Result<()> {
        if !self.is_running() {
            bail!("Call .run() before .set_activity().");
        }

        self.tx
            .send(IPCCommand::SetActivity {
                details: details.to_string(),
                state: state.to_string(),
            })
            .await?;

        Ok(())
    }

    /// Clears a previously-set Discord Rich Presence activity. Prefer this function over
    /// close() if the current process is not being exited with the presence toggle.
    pub async fn clear_activity(&self) -> Result<()> {
        if self.is_running() {
            self.tx.send(IPCCommand::ClearActivity).await?;
        }

        Ok(())
    }

    const CLOSE_COOLDOWN_MS: u64 = 200;

    /// Closes the current session of Rich Presence activity.
    ///
    /// NOTE: Always try to use one activity per session as Discord might sometimes behave
    /// weirdly with session closes; this was only found noticeable if run() and close() were
    /// repeatedly called.
    pub async fn close(&mut self) -> Result<()> {
        if !self.running.swap(false, Ordering::SeqCst) {
            return Ok(());
        }

        self.tx.send(IPCCommand::Close).await?;

        if let Some(handle) = self.handle.take() {
            match timeout(Duration::from_secs(2), handle).await {
                Ok(result) => result??,
                Err(_) => eprintln!("DiscordIPC close() timed out."),
            }
        }

        sleep(Duration::from_millis(Self::CLOSE_COOLDOWN_MS)).await;

        Ok(())
    }
}

/*
 *
 * Command-specific functions
 *
 */

async fn send_activity(
    socket: &mut DiscordIPCSocket,
    details: &str,
    state: &str,
    timestamp: u64,
) -> Result<()> {
    let json = json!({
        "cmd": "SET_ACTIVITY",
        "args": {
            "pid": std::process::id(),
            "activity": {
                "details": details,
                "state": state,
                "timestamps": { "start": timestamp }
            }
        },
        "nonce": Uuid::new_v4().to_string()
    })
    .to_string();

    let packed = pack(1, json.len() as u32);
    socket.write(&packed).await?;
    socket.write(json.as_bytes()).await?;
    Ok(())
}

async fn clear_activity(socket: &mut DiscordIPCSocket) -> Result<()> {
    let json = json!({
        "cmd": "SET_ACTIVITY",
        "args": {
            "pid": std::process::id(),
            "activity": null
        },
        "nonce": Uuid::new_v4().to_string()
    })
    .to_string();

    let packed = pack(1, json.len() as u32);
    socket.write(&packed).await?;
    socket.write(json.as_bytes()).await?;
    Ok(())
}

/*
 *
 * Blocking wrapper
 *
 */

/// Synchronous wrapper around [`DiscordIPC`] for use in non-async contexts.
/// All operations block on an internal Tokio runtime.
#[derive(Debug)]
pub struct DiscordIPCSync {
    inner: DiscordIPC,
    rt: Runtime,
}

impl DiscordIPCSync {
    /// Creates a new synchronous Discord IPC client with the given client ID.
    pub fn new(client_id: &str) -> Result<Self> {
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let inner = DiscordIPC::new(client_id);
        Ok(Self { inner, rt })
    }

    /// Returns the Discord client ID that has been used to initialize this IPC client.
    pub fn client_id(&self) -> String {
        self.inner.client_id()
    }

    /// Run the IPC client. Must be called before any `set_activity()` calls.
    pub fn run(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.run())
    }

    /// Waits for the primary IPC task to finish.
    pub fn wait(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.wait())
    }

    /// Sets/updates the Discord Rich Presence activity.
    /// `run()` must be called prior to calling this.
    pub fn set_activity(&self, details: &str, state: &str) -> Result<()> {
        self.rt.block_on(self.inner.set_activity(details, state))
    }

    /// Clears the current Discord Rich Presence activity without closing the session.
    pub fn clear_activity(&self) -> Result<()> {
        self.rt.block_on(self.inner.clear_activity())
    }

    /// Closes the current session of Rich Presence activity.
    /// After this, you can safely call `run()` again.
    pub fn close(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.close())
    }
}
