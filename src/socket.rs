// SPDX-License-Identifier: MIT

use anyhow::{Result, bail};
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

#[cfg(target_family = "windows")]
type ReadHalfCore = ReadHalf<NamedPipeClient>;
#[cfg(target_family = "windows")]
type WriteHalfCore = WriteHalf<NamedPipeClient>;

#[cfg(target_family = "unix")]
type ReadHalfCore = OwnedReadHalf;
#[cfg(target_family = "unix")]
type WriteHalfCore = OwnedWriteHalf;

pub(crate) struct Frame {
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
pub(crate) struct DiscordIPCSocket {
    readhalf: Arc<Mutex<ReadHalfCore>>,
    writehalf: Arc<Mutex<WriteHalfCore>>,
}

impl DiscordIPCSocket {
    #[cfg(target_os = "windows")]
    async fn get_socket() -> Result<(ReadHalfCore, WriteHalfCore)> {
        let path = match get_pipe_path() {
            Some(p) => p,
            None => return bail!("Pipe not found."),
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

    pub(crate) async fn new() -> Result<Self> {
        let result = Self::get_socket().await;

        match result {
            Ok((readhalf, writehalf)) => {
                Ok(Self {
                    readhalf: Arc::new(Mutex::new(readhalf)),
                    writehalf: Arc::new(Mutex::new(writehalf)),
                })
            }
            Err(e) => bail!("Error while creating new IPC client: {e}"),
        }
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> Result<()> {
        acquire!(&self.readhalf, stream);
        stream.read_exact(buffer).await?;
        Ok(())
    }

    pub(crate) async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        acquire!(&self.writehalf, stream);
        stream.write_all(buffer).await?;
        Ok(())
    }

    pub(crate) async fn read_frame(&mut self) -> Result<Frame> {
        let mut header = [0u8; 8];
        self.read(&mut header).await?;

        let opcode = u32::from_le_bytes(header[0..4].try_into()?);
        let len = u32::from_le_bytes(header[4..8].try_into()?) as usize;

        let mut body = vec![0u8; len];
        self.read(&mut body).await?;

        Ok(Frame { opcode, body })
    }
}
