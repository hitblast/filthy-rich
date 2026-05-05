use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bytes::{Bytes, BytesMut};
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

#[cfg(target_family = "unix")]
use std::env::var;

use crate::errors::{DiscordSockError, InnerParsingError};
use crate::types::{ActivityCommand, PresenceHandshake, SendableActivity};

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
    pub body: Bytes,
}

//  Defaults to 16 Mib (just an assumption), this can be changed later
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

macro_rules! acquire {
    ($s:expr, $x:ident) => {
        let mut $x = $s.lock().await;
    };
}

#[cfg(target_family = "unix")]
fn add_unix_candidates(candidates: &mut HashSet<String>, base_dir: &str) {
    candidates.insert(format!("{base_dir}/discord-ipc-")); // normal sane discord path
    candidates.insert(format!(
        "{base_dir}/app/com.discordapp.Discord/discord-ipc-"
    )); // for flatpak

    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("snap.discord") {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_dir() {
                            candidates.insert(format!("{base_dir}/{name}/discord-ipc-"));
                        }
                    }
                }
            }
        }
    }
}

fn get_pipe_path() -> Option<PathBuf> {
    let mut candidates = HashSet::new();

    #[cfg(target_os = "windows")]
    candidates.insert(r"\\?\pipe\discord-ipc-".to_string());

    #[cfg(target_family = "unix")]
    {
        add_unix_candidates(&mut candidates, "/tmp");

        if let Ok(runtime_dir) = var("TMPDIR") {
            add_unix_candidates(&mut candidates, &runtime_dir);
        }

        if let Ok(runtime_dir) = var("XDG_RUNTIME_DIR") {
            add_unix_candidates(&mut candidates, &runtime_dir);
        }

        if let Ok(uid) = var("UID") {
            add_unix_candidates(&mut candidates, &format!("/run/user/{uid}"));
        }
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
    readhalf: Mutex<ReadHalfCore>,
    writehalf: Mutex<WriteHalfCore>,
}

impl DiscordSock {
    #[cfg(target_os = "windows")]
    async fn get_socket() -> Result<(ReadHalfCore, WriteHalfCore), DiscordSockError> {
        let path = match get_pipe_path() {
            Some(p) => p,
            None => return Err(DiscordSockError::PipeNotFound),
        };
        if let Ok(client) = ClientOptions::new().open(&path) {
            let (read_half, write_half) = tokio::io::split(client);
            return Ok((read_half, write_half));
        }

        Err(DiscordSockError::PipeConnectionFailed)
    }

    #[cfg(target_family = "unix")]
    async fn get_socket() -> Result<(ReadHalfCore, WriteHalfCore), DiscordSockError> {
        let path = match get_pipe_path() {
            Some(p) => p,
            None => return Err(DiscordSockError::PipeNotFound),
        };

        if let Ok(socket) = UnixStream::connect(&path).await {
            return Ok(socket.into_split());
        }

        Err(DiscordSockError::PipeConnectionFailed)
    }

    pub async fn new() -> Result<Self, DiscordSockError> {
        let result = Self::get_socket().await;

        match result {
            Ok((readhalf, writehalf)) => Ok(Self {
                readhalf: Mutex::new(readhalf),
                writehalf: Mutex::new(writehalf),
            }),
            Err(e) => Err(e),
        }
    }

    async fn read_exact(&self, buffer: &mut [u8]) -> Result<(), DiscordSockError> {
        acquire!(&self.readhalf, stream);
        stream.read_exact(buffer).await?;
        Ok(())
    }

    pub async fn write<T: AsRef<[u8]>>(&self, buffer: T) -> Result<(), DiscordSockError> {
        acquire!(&self.writehalf, stream);
        stream.write_all(buffer.as_ref()).await?;
        Ok(())
    }

    pub async fn read_frame(&self) -> Result<Frame, DiscordSockError> {
        let mut header = [0u8; 8];
        self.read_exact(&mut header).await?;

        let opcode = u32::from_le_bytes(header[0..4].try_into()?);
        let len = u32::from_le_bytes(header[4..8].try_into()?) as usize;

        if len > MAX_FRAME_SIZE {
            return Err(DiscordSockError::PayloadTooLarge {
                size: len,
                max: MAX_FRAME_SIZE,
            });
        }

        let mut body = BytesMut::with_capacity(len);
        body.resize(len, 0);
        self.read_exact(&mut body).await?;

        Ok(Frame {
            opcode,
            body: body.freeze(),
        })
    }

    pub async fn send_frame<T: AsRef<[u8]>>(
        &self,
        opcode: u32,
        body: T,
    ) -> Result<(), DiscordSockError> {
        let body = body.as_ref();
        if body.len() > u32::MAX as usize {
            return Err(DiscordSockError::PayloadTooLarge {
                size: body.len(),
                max: u32::MAX as usize,
            });
        }
        let mut buf = BytesMut::with_capacity(8 + body.len());

        buf.extend_from_slice(&opcode.to_le_bytes());
        buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
        buf.extend_from_slice(body);

        self.write(buf.freeze()).await?;
        Ok(())
    }

    pub async fn close(self) -> Result<(), DiscordSockError> {
        #[cfg(target_family = "unix")]
        {
            let mut write = self.writehalf.lock().await;
            write.flush().await?;
            write.shutdown().await?;
        }

        // for future me: in case you've forgotten how it works, essentially on Windows, since
        // we return Ok(()) here, the struct instance is dropped, hence dropping the NamedPipes
        // in the process - closing the socket.

        Ok(())
    }
}

impl DiscordSock {
    pub(crate) async fn send_activity(
        &mut self,
        activity: SendableActivity,
        session_start: u64,
    ) -> Result<(), DiscordSockError> {
        let activity = activity.populate_time(session_start)?;
        let cmd = ActivityCommand::new_with(Some(activity));

        self.send_cmd(cmd).await?;
        Ok(())
    }

    pub(crate) async fn clear_activity(&mut self) -> Result<(), DiscordSockError> {
        let cmd = ActivityCommand::new_with(None);
        self.send_cmd(cmd).await?;
        Ok(())
    }

    pub(crate) async fn do_handshake(&mut self, client_id: &str) -> Result<(), DiscordSockError> {
        let handshake = PresenceHandshake { v: 1, client_id };
        let json = serde_json::to_string(&handshake).map_err(InnerParsingError::from)?;

        self.send_frame(0, json).await?;
        Ok(())
    }

    async fn send_cmd(&mut self, cmd: ActivityCommand) -> Result<(), DiscordSockError> {
        let cmd = cmd.to_json()?;
        self.send_frame(1, cmd).await?;
        Ok(())
    }
}
