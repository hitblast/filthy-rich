use anyhow::{Result, bail};
use serde_json::json;
use std::collections::HashSet;
use std::env::var;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(target_family = "unix")]
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

#[cfg(target_family = "unix")]
use tokio::net::UnixStream;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

#[cfg(target_family = "windows")]
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient},
};

use crate::types::{Activity, ActivityCommand, ActivityPayload, ButtonPayload, TimestampPayload};
use crate::utils::get_current_timestamp;

#[cfg(target_family = "windows")]
type ReadHalfCore = ReadHalf<NamedPipeClient>;
#[cfg(target_family = "windows")]
type WriteHalfCore = WriteHalf<NamedPipeClient>;

#[cfg(target_family = "unix")]
type ReadHalfCore = OwnedReadHalf;
#[cfg(target_family = "unix")]
type WriteHalfCore = OwnedWriteHalf;

pub struct Frame {
    pub opcode: u32,
    pub body: Vec<u8>,
}

macro_rules! acquire {
    ($s:expr, $x:ident) => {
        let mut $x = $s.lock().await;
    };
}

fn get_pipe_path() -> Option<PathBuf> {
    let mut candidates = HashSet::new();

    #[cfg(target_os = "windows")]
    candidates.insert(r"\\?\pipe\discord-ipc-".to_string());

    #[cfg(target_family = "unix")]
    candidates.insert("/tmp/discord-ipc-".to_string());

    if let Ok(runtime_dir) = var("TMPDIR") {
        candidates.insert(runtime_dir + "/discord-ipc-");
    }

    if let Ok(runtime_dir) = var("XDG_RUNTIME_DIR") {
        candidates.insert(runtime_dir.clone() + "/app/com.discordapp.Discord/discord-ipc-");
        candidates.insert(runtime_dir + "/discord-ipc-");
    }

    for i in 0..10 {
        for p in &candidates {
            let path: String = format!("{}{}", p, i);

            if Path::new(&path).exists() {
                return Some(Path::new(&path).to_path_buf());
            }
        }
    }

    None
}

#[derive(Debug)]
pub struct DiscordSock {
    readhalf: Arc<Mutex<ReadHalfCore>>,
    writehalf: Arc<Mutex<WriteHalfCore>>,
}

impl DiscordSock {
    #[cfg(target_os = "windows")]
    async fn get_socket() -> Result<(ReadHalfCore, WriteHalfCore)> {
        let path = match get_pipe_path() {
            Some(p) => p,
            None => bail!("Pipe not found."),
        };
        if let Ok(client) = ClientOptions::new().open(&path) {
            let (read_half, write_half) = tokio::io::split(client);
            return Ok((read_half, write_half));
        }

        bail!("Could not connect to pipe.")
    }

    #[cfg(target_family = "unix")]
    async fn get_socket() -> Result<(ReadHalfCore, WriteHalfCore)> {
        let path = match get_pipe_path() {
            Some(p) => p,
            None => bail!("Pipe not found."),
        };

        if let Ok(socket) = UnixStream::connect(&path).await {
            return Ok(socket.into_split());
        }

        bail!("Could not connect to pipe.")
    }

    pub async fn new() -> Result<Self> {
        let result = Self::get_socket().await;

        match result {
            Ok((readhalf, writehalf)) => Ok(Self {
                readhalf: Arc::new(Mutex::new(readhalf)),
                writehalf: Arc::new(Mutex::new(writehalf)),
            }),
            Err(e) => bail!("Error while creating new IPC RPC socket: {e}"),
        }
    }

    async fn read(&self, buffer: &mut [u8]) -> Result<()> {
        acquire!(&self.readhalf, stream);
        stream.read_exact(buffer).await?;
        Ok(())
    }

    pub async fn write<T: AsRef<[u8]>>(&self, buffer: T) -> Result<()> {
        acquire!(&self.writehalf, stream);
        stream.write_all(buffer.as_ref()).await?;
        Ok(())
    }

    pub async fn read_frame(&self) -> Result<Frame> {
        let mut header = [0u8; 8];
        self.read(&mut header).await?;

        let opcode = u32::from_le_bytes(header[0..4].try_into()?);
        let len = u32::from_le_bytes(header[4..8].try_into()?) as usize;

        let mut body = vec![0u8; len];
        self.read(&mut body).await?;

        Ok(Frame { opcode, body })
    }

    pub async fn send_frame<T: AsRef<[u8]>>(&self, opcode: u32, body: T) -> Result<()> {
        let mut header = Vec::with_capacity(8);
        header.extend_from_slice(&opcode.to_le_bytes());
        header.extend_from_slice(&(body.as_ref().len() as u32).to_le_bytes());

        self.write(&header).await?;
        self.write(body).await?;
        Ok(())
    }

    pub async fn close(self) -> Result<()> {
        // read half: nothing to shutdown, just drop the guard
        {
            let _read = self.readhalf.lock().await;
        }

        // write half: can shutdown
        let mut write = self.writehalf.lock().await;

        #[cfg(target_family = "unix")]
        {
            write.flush().await?;
            write.shutdown().await?;
        }

        // on Windows, dropping the halves closes the NamedPipe

        Ok(())
    }
}

impl DiscordSock {
    pub(crate) async fn send_activity(
        &mut self,
        activity: Activity,
        session_start: u64,
    ) -> Result<()> {
        let current_t = get_current_timestamp()?;
        let end_timestamp = activity.duration.map(|d| current_t + d.as_secs());

        let assets = if activity.large_image.is_some() || activity.small_image.is_some() {
            Some(crate::types::AssetsPayload {
                large_image: activity.large_image,
                large_text: activity.large_text,
                large_url: activity.large_url,
                small_image: activity.small_image,
                small_text: activity.small_text,
                small_url: activity.small_url,
            })
        } else {
            None
        };

        let buttons: Option<Vec<ButtonPayload>> = activity.buttons.map(|btns| {
            btns.into_iter()
                .map(|f| ButtonPayload {
                    label: f.0,
                    url: f.1,
                })
                .collect()
        });

        let cmd = ActivityCommand::new_with(Some(ActivityPayload {
            name: activity.name,
            r#type: activity.activity_type.into(),
            created_at: current_t,
            status_display_type: activity.status_display_type.map(|f| f.into()),
            details: activity.details,
            state: activity.state,
            timestamps: TimestampPayload {
                start: session_start,
                end: end_timestamp,
            },
            assets,
            buttons,
        }));

        self.send_cmd(cmd).await?;
        Ok(())
    }

    pub(crate) async fn clear_activity(&mut self) -> Result<()> {
        let cmd = ActivityCommand::new_with(None);
        self.send_cmd(cmd).await?;
        Ok(())
    }

    pub(crate) async fn do_handshake(&mut self, client_id: &str) -> Result<()> {
        let handshake = json!({ "v": 1, "client_id": client_id }).to_string();
        self.send_frame(0, handshake).await?;
        Ok(())
    }

    async fn send_cmd(&mut self, cmd: ActivityCommand) -> Result<()> {
        let cmd = cmd.to_json()?;
        self.send_frame(1, cmd).await?;
        Ok(())
    }
}
