// SPDX-License-Identifier: MIT

use std::{sync::Arc, time::Duration};

use anyhow::{Result, bail};
use tokio::{
    runtime::{Builder, Runtime},
    sync::Mutex,
    task::JoinHandle,
    time::sleep,
};
use uuid::Uuid;

use crate::{
    socket::DiscordIPCSocket,
    utils::{get_current_timestamp, pack},
};

/// Blocking representation of DiscordIPC.
#[derive(Debug)]
pub struct DiscordIPCSync {
    inner: DiscordIPC,
    rt: Runtime,
    ipc_task: Option<JoinHandle<Result<()>>>,
}

impl DiscordIPCSync {
    /// Given a client ID, create a new `DiscordIPCSync` instance.
    /// Needs to have Discord running for successful execution.
    ///
    /// NOTE: Essentially a `DiscordIPC` instance but with blocking I/O.
    pub fn new(client_id: &str) -> Result<Self> {
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let inner = rt.block_on(DiscordIPC::new(client_id))?;
        Ok(Self {
            inner,
            rt,
            ipc_task: None,
        })
    }

    /// Blocking iteration of `DiscordIPC::run`
    pub fn run(&mut self) -> Result<()> {
        if self.ipc_task.is_some() {
            bail!(".run() called multiple times over DiscordIPC.")
        }

        let handle = self.rt.block_on(self.inner.run())?;
        self.ipc_task = Some(handle);

        Ok(())
    }

    /// Convenience function for indefinitely running the Discord IPC message receiver loop; must use *after* `DiscordIPCSync::run`.
    pub fn wait(&mut self) -> Result<()> {
        if let Some(handle) = self.ipc_task.take() {
            self.rt.block_on(handle)??;
        }
        Ok(())
    }

    /// Blocking iteration of `DiscordIPC::set_activity`
    pub fn set_activity(&mut self, details: &str, state: &str) -> Result<()> {
        self.rt.block_on(self.inner.set_activity(details, state))
    }
}

/// Basic Discord rich presence IPC implementation.
#[derive(Debug, Clone)]
pub struct DiscordIPC {
    sock: Arc<Mutex<Option<DiscordIPCSocket>>>,
    timestamp: u64,
    client_id: String,
}

impl DiscordIPC {
    async fn send_json(&self, json: String, opcode: u32) -> Result<()> {
        let bytes = json.as_bytes();
        let mut sock_guard = self.sock.lock().await;

        if let Some(sock) = sock_guard.as_mut() {
            let packed = pack(opcode, bytes.len() as u32)?;
            sock.write(&packed).await?;
            sock.write(bytes).await?;
        }

        Ok(())
    }

    /// Given a client ID, create a new `DiscordIPC` instance.
    /// Needs to have Discord running for successful execution.
    pub async fn new(client_id: &str) -> Result<Self> {
        Ok(Self {
            sock: Arc::new(Mutex::new(None)),
            timestamp: get_current_timestamp()?,
            client_id: client_id.to_string(),
        })
    }

    async fn connect(&mut self) -> Result<()> {
        self.sock = Arc::new(Mutex::new(Some(DiscordIPCSocket::new().await?)));
        Ok(())
    }

    async fn handshake(&mut self) -> Result<()> {
        let json = format!(r#"{{"v":1,"client_id":"{}"}}"#, self.client_id);
        self.send_json(json, 0u32).await?;

        Ok(())
    }

    async fn wait_for_ready(&mut self) -> Result<()> {
        let mut sock_guard = self.sock.lock().await;

        loop {
            if let Some(sock) = sock_guard.as_mut() {
                let frame = sock.read_frame().await?;

                if frame.opcode == 1 && frame.body.windows(5).any(|w| w == b"READY") {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Starts off the connection with Discord. This includes performing a handshake, waiting for READY and
    /// starting the IPC response loop.
    pub async fn run(&mut self) -> Result<JoinHandle<Result<()>>> {
        let mut this = self.clone();

        let handle = tokio::spawn(async move {
            let mut backoff = 1;
            loop {
                if let Err(_) = this.connect().await {
                    sleep(Duration::from_secs(backoff)).await;
                    continue;
                }
                if let Err(_) = this.handshake().await {
                    sleep(Duration::from_secs(backoff)).await;
                    continue;
                }
                if let Err(_) = this.wait_for_ready().await {
                    sleep(Duration::from_secs(backoff)).await;
                    continue;
                }
                loop {
                    let mut sock_guard = this.sock.lock().await;
                    if let Some(s) = sock_guard.as_mut() {
                        let frame = match s.read_frame().await {
                            Ok(f) => f,
                            Err(_) => break,
                        };
                        match frame.opcode {
                            3 => {
                                let pack = pack(frame.opcode, frame.body.len() as u32)?;
                                s.write(&pack).await?;
                                s.write(&frame.body).await?;
                            }
                            2 => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }

                this.sock = Arc::new(Mutex::new(None));

                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(4);
            }
        });
        Ok(handle)
    }

    /// Sets a tiny Discord rich presence activity.
    pub async fn set_activity(&mut self, details: &str, state: &str) -> Result<()> {
        let pid = std::process::id();
        let uuid = Uuid::new_v4();

        let json = format!(
            r#"
{{
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
}}
"#,
            pid, details, state, self.timestamp, uuid
        );

        self.send_json(json, 1u32).await?;
        Ok(())
    }
}
