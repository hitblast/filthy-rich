// SPDX-License-Identifier: MIT

use std::time::Duration;

use anyhow::{Result, bail};
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc,
    task::JoinHandle,
    time::sleep,
};
use uuid::Uuid;

use crate::{
    socket::DiscordIPCSocket,
    utils::{get_current_timestamp, pack},
};

/// Commands sent to the IPC task.
#[derive(Debug)]
enum IPCCommand {
    SetActivity { details: String, state: String },
}

/// Async Discord IPC client.
#[derive(Debug, Clone)]
pub struct DiscordIPC {
    tx: mpsc::Sender<IPCCommand>,
    client_id: String,
    timestamp: u64,
}

impl DiscordIPC {
    /// Create a new IPC instance (does NOT start connection).
    /// To start a connection and run the client, use `.run()`.
    pub async fn new(client_id: &str) -> Result<Self> {
        let (tx, _rx) = mpsc::channel(32);

        Ok(Self {
            tx,
            client_id: client_id.to_string(),
            timestamp: get_current_timestamp()?,
        })
    }

    /// Connect, handshake, wait for READY and  start the IPC client.
    pub async fn run(&mut self) -> Result<JoinHandle<Result<()>>> {
        let (tx, mut rx) = mpsc::channel::<IPCCommand>(32);
        self.tx = tx;

        let client_id = self.client_id.clone();
        let timestamp = self.timestamp;

        let handle = tokio::spawn(async move {
            let mut backoff = 1;
            let mut last_activity: Option<(String, String)> = None;

            loop {
                // initial connect
                let mut socket = match DiscordIPCSocket::new().await {
                    Ok(s) => s,
                    Err(_) => {
                        sleep(Duration::from_secs(backoff)).await;
                        continue;
                    }
                };

                // handshake
                let handshake = format!(r#"{{"v":1,"client_id":"{}"}}"#, client_id);

                let packed = pack(0, handshake.len() as u32)?;
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

                    if frame.opcode == 1 && frame.body.windows(5).any(|w| w == b"READY") {
                        break;
                    }
                }

                // replay activity after socket
                if let Some((details, state)) = &last_activity {
                    let _ = send_activity(&mut socket, details, state, timestamp).await;
                }

                backoff = 1;

                // main loop
                loop {
                    tokio::select! {
                        Some(cmd) = rx.recv() => {
                            match cmd {
                                IPCCommand::SetActivity { details, state } => {
                                    last_activity = Some((details.clone(), state.clone()));

                                    if send_activity(&mut socket, &details, &state, timestamp).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }

                        frame = socket.read_frame() => {
                            match frame {
                                Ok(frame) => {
                                    match frame.opcode {
                                        // PING
                                        3 => {
                                            let packed = pack(frame.opcode, frame.body.len() as u32)?;
                                            socket.write(&packed).await?;
                                            socket.write(&frame.body).await?;
                                        }
                                        // CLOSE
                                        2 => break,
                                        _ => {}
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                }

                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(4);
            }
        });

        Ok(handle)
    }

    /// Sets the Discord Rich presence activity.
    pub async fn set_activity(&self, details: &str, state: &str) -> Result<()> {
        self.tx
            .send(IPCCommand::SetActivity {
                details: details.to_string(),
                state: state.to_string(),
            })
            .await?;

        Ok(())
    }
}

/// Helper to send activity JSON.
async fn send_activity(
    socket: &mut DiscordIPCSocket,
    details: &str,
    state: &str,
    timestamp: u64,
) -> Result<()> {
    let pid = std::process::id();
    let uuid = Uuid::new_v4();

    let json = format!(
        r#"{{
    "cmd":"SET_ACTIVITY",
    "args": {{
        "pid": {},
        "activity": {{
            "details":"{}",
            "state":"{}",
            "timestamps": {{
                "start": {}
            }}
        }}
    }},
    "nonce":"{}"
}}"#,
        pid, details, state, timestamp, uuid
    );

    let packed = pack(1, json.len() as u32)?;
    socket.write(&packed).await?;
    socket.write(json.as_bytes()).await?;

    Ok(())
}

/// Blocking wrapper around DiscordIPC.
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

    pub fn run(&mut self) -> Result<()> {
        if self.ipc_task.is_some() {
            bail!("run() called multiple times");
        }

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
}
