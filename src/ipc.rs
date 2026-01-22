use anyhow::Result;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::{socket::DiscordIPCSocket, utils::pack};

/// Basic Discord rich presence IPC implementation.
/// See the docs: https://docs.rs/crate/filthy-rich/latest
pub struct DiscordIPC {
    sock: DiscordIPCSocket,
    ipc_task: Option<JoinHandle<Result<()>>>,
    client_id: String,
}

impl DiscordIPC {
    /// Given a client ID, create a new DiscordIPC instance.
    /// Needs to have Discord running for successful execution.
    pub async fn new_from(client_id: &str) -> Result<Self> {
        let sock = DiscordIPCSocket::new().await?;

        Ok(Self {
            sock,
            ipc_task: None,
            client_id: client_id.to_string(),
        })
    }

    /// Bare-bones implementation of handshake with the Discord IPC.
    /// Use `.run()` instead.
    pub async fn handshake(&mut self) -> Result<()> {
        let json = format!(r#"{{"v":1,"client_id":"{}"}}"#, self.client_id);
        let bytes = json.as_bytes();

        let pack = pack(0u32, bytes.len() as u32)?;
        self.sock.write(&pack).await?;
        self.sock.write(bytes).await?;

        Ok(())
    }

    /// Look out for READY in socket frames. Use `.run()` instead.
    pub async fn wait_for_ready(&mut self) -> Result<()> {
        loop {
            let frame = self.sock.read_frame().await?;

            if frame.opcode == 1 && frame.body.windows(5).any(|w| w == b"READY") {
                break;
            }
        }
        Ok(())
    }

    /// Convenience function for performing handshake, waiting for READY opcode
    /// and handling the IPC response loop.
    pub async fn run(&mut self) -> Result<()> {
        if self.ipc_task.is_some() {
            return Ok(());
        }

        self.handshake().await?;
        self.wait_for_ready().await?;

        let mut sock = self.sock.clone();
        self.ipc_task = Some(tokio::spawn(async move { sock.handle_ipc().await }));

        Ok(())
    }

    /// Waits for response from IPC task; can be used to run the client indefinitely.
    pub async fn wait(&mut self) -> Result<()> {
        if let Some(handle) = &mut self.ipc_task {
            match handle.await {
                Ok(res) => res?,
                Err(e) if e.is_cancelled() => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    /// Sets a tiny Discord rich presence activity.
    pub async fn set_activity(&mut self, top: &str, bottom: &str) -> Result<()> {
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
            "state":"{}"
        }}
    }},
    "nonce":"{}"
}}
"#,
            pid, top, bottom, uuid
        );

        let bytes = json.as_bytes();
        let pack = pack(1u32, bytes.len() as u32)?;

        self.sock.write(&pack).await?;
        self.sock.write(bytes).await?;
        Ok(())
    }
}
