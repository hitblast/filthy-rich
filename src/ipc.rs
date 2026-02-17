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
    time::sleep,
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
    Close,
}

#[derive(Debug, Deserialize)]
struct RpcFrame {
    cmd: Option<String>,
    evt: Option<String>,
    data: Option<serde_json::Value>,
}

/// Primary struct for you to set and update Discord Rich Presences with.
#[derive(Debug, Clone)]
pub struct DiscordIPC {
    tx: Sender<IPCCommand>,
    client_id: String,
    running: Arc<AtomicBool>,
}

impl DiscordIPC {
    pub async fn new(client_id: &str) -> Result<Self> {
        let (tx, _rx) = mpsc::channel(32);

        Ok(Self {
            tx,
            client_id: client_id.to_string(),
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// The Discord client ID that has been used to initialize this IPC client instance.
    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    /// Run the client.
    /// Returns a `JoinHandle<anyhow::Result<()>>` for management.
    /// NOTE: Must be called before any .set_activity() calls.
    pub async fn run(&mut self) -> Result<JoinHandle<Result<()>>> {
        if self.running.swap(true, Ordering::SeqCst) {
            bail!(
                "Cannot run multiple instances of .run() for DiscordIPC, or when a session is still closing."
            )
        }

        let (tx, mut rx) = mpsc::channel::<IPCCommand>(32);
        self.tx = tx;
        let client_id = self.client_id.clone();

        let running = self.running.clone();

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
                                IPCCommand::Close => {
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
                                        socket.write(&packed).await?;
                                        socket.write(&frame.body).await?;
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

        Ok(handle)
    }

    /// Sets/updates the Discord Rich presence activity.
    /// NOTE: .run() must be executed prior to calling this.
    pub async fn set_activity(&self, details: &str, state: &str) -> Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            bail!("Call .run() before .set_activity() execution.");
        }

        self.tx
            .send(IPCCommand::SetActivity {
                details: details.to_string(),
                state: state.to_string(),
            })
            .await?;
        Ok(())
    }

    /// Closes the current session of Rich Presence activity.
    pub async fn close(&self) -> Result<()> {
        self.tx.send(IPCCommand::Close).await?;
        Ok(())
    }
}

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

/// Blocking wrapper
#[derive(Debug)]
pub struct DiscordIPCSync {
    inner: DiscordIPC,
    rt: Runtime,
    ipc_task: Option<JoinHandle<Result<()>>>,
}

impl DiscordIPCSync {
    pub fn new(client_id: &str) -> Result<Self> {
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let inner = rt.block_on(DiscordIPC::new(client_id))?;
        Ok(Self {
            inner,
            rt,
            ipc_task: None,
        })
    }

    pub fn client_id(&self) -> String {
        self.inner.client_id()
    }

    pub fn run(&mut self) -> Result<()> {
        let handle = self.rt.block_on(self.inner.run())?;
        self.ipc_task = Some(handle);
        Ok(())
    }

    pub fn wait(&mut self) -> Result<()> {
        if let Some(handle) = self.ipc_task.take() {
            self.rt.block_on(handle)??;
        }
        Ok(())
    }

    pub fn set_activity(&self, details: &str, state: &str) -> Result<()> {
        self.rt.block_on(self.inner.set_activity(details, state))
    }

    pub fn close(&self) -> Result<()> {
        self.rt.block_on(self.inner.close())
    }
}

impl Drop for DiscordIPCSync {
    fn drop(&mut self) {
        if let Some(handle) = self.ipc_task.take() {
            let _ = self.rt.block_on(self.inner.close());
            let _ = self.rt.block_on(handle);
        }
    }
}
