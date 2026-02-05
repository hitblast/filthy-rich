// SPDX-License-Identifier: MIT

use std::thread::{self, JoinHandle};

use anyhow::{Result, bail};
use uuid::Uuid;

use crate::{
    socket::DiscordIPCSocket,
    utils::{get_current_timestamp_unix, pack},
};

/// Basic Discord rich presence IPC implementation.
/// See the docs: <https://docs.rs/crate/filthy-rich/latest>
pub struct DiscordIPC {
    sock: DiscordIPCSocket,
    ipc_task: Option<JoinHandle<Result<()>>>,
    timestamp: u64,
    client_id: String,
}

impl DiscordIPC {
    fn send_json(&mut self, json: String, opcode: u32) -> Result<()> {
        let bytes = json.as_bytes();

        let packed = pack(opcode, bytes.len() as u32)?;
        self.sock.write(&packed)?;
        self.sock.write(bytes)?;

        Ok(())
    }

    /// Given a client ID, create a new DiscordIPC instance.
    /// Needs to have Discord running for successful execution.
    pub fn new_from(client_id: &str) -> Result<Self> {
        let sock = DiscordIPCSocket::new()?;

        Ok(Self {
            sock,
            ipc_task: None,
            timestamp: get_current_timestamp_unix()?,
            client_id: client_id.to_string(),
        })
    }

    /// Bare-bones implementation of handshake with the Discord IPC.
    /// Use `.run()` instead.
    pub fn handshake(&mut self) -> Result<()> {
        let json = format!(r#"{{"v":1,"client_id":"{}"}}"#, self.client_id);
        self.send_json(json, 0u32)?;

        Ok(())
    }

    /// Look out for READY in socket frames. Use `.run()` instead.
    pub fn wait_for_ready(&mut self) -> Result<()> {
        loop {
            let frame = self.sock.read_frame()?;

            if frame.opcode == 1 && frame.body.windows(5).any(|w| w == b"READY") {
                break;
            }
        }
        Ok(())
    }

    /// Convenience function for performing handshake, waiting for READY opcode
    /// and handling the IPC response loop.
    pub fn run(&mut self) -> Result<()> {
        if self.ipc_task.is_some() {
            return Ok(());
        }

        self.handshake()?;
        self.wait_for_ready()?;

        let mut sock = self.sock.clone();
        self.ipc_task = Some(thread::spawn(move || sock.handle_ipc()));

        Ok(())
    }

    /// Waits for response from IPC task; can be used to run the client indefinitely.
    pub fn wait(&mut self) -> Result<()> {
        if let Some(handle) = self.ipc_task.take() {
            match handle.join() {
                Ok(res) => res?,
                Err(e) => bail!("Thread panicked: {e:?}"),
            }
        }
        Ok(())
    }

    /// Sets a tiny Discord rich presence activity.
    pub fn set_activity(&mut self, details: &str, state: &str) -> Result<()> {
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

        self.send_json(json, 1u32)?;
        Ok(())
    }
}
