// SPDX-License-Identifier: MIT

use anyhow::{Result, bail};
use tokio::{
    runtime::{Builder, Runtime},
    task::JoinHandle,
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
    sock: DiscordIPCSocket,
    timestamp: u64,
    client_id: String,
}

impl DiscordIPC {
    async fn send_json(&mut self, json: String, opcode: u32) -> Result<()> {
        let bytes = json.as_bytes();

        let packed = pack(opcode, bytes.len() as u32)?;
        self.sock.write(&packed).await?;
        self.sock.write(bytes).await?;

        Ok(())
    }

    /// Given a client ID, create a new `DiscordIPC` instance.
    /// Needs to have Discord running for successful execution.
    pub async fn new(client_id: &str) -> Result<Self> {
        let sock = DiscordIPCSocket::new().await?;

        Ok(Self {
            sock,
            timestamp: get_current_timestamp()?,
            client_id: client_id.to_string(),
        })
    }

    async fn handshake(&mut self) -> Result<()> {
        let json = format!(r#"{{"v":1,"client_id":"{}"}}"#, self.client_id);
        self.send_json(json, 0u32).await?;

        Ok(())
    }

    async fn wait_for_ready(&mut self) -> Result<()> {
        loop {
            let frame = self.sock.read_frame().await?;

            if frame.opcode == 1 && frame.body.windows(5).any(|w| w == b"READY") {
                break;
            }
        }
        Ok(())
    }

    /// Starts off the connection with Discord. This includes performing a handshake, waiting for READY and
    /// starting the IPC response loop.
    pub async fn run(&mut self) -> Result<JoinHandle<Result<()>>> {
        self.handshake().await?;
        self.wait_for_ready().await?;

        let mut sock = self.sock.clone();
        let handle = tokio::spawn(async move { sock.handle_ipc().await });

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
