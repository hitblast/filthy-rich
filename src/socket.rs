// SPDX-License-Identifier: MIT

use anyhow::{Context, Result, bail};
use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Mutex,
};

use crate::utils::{get_pipe_path, pack};

pub(crate) struct Frame {
    pub opcode: u32,
    pub body: Vec<u8>,
}

macro_rules! acquire {
    ($s:expr, $x:ident) => {
        let mut $x = $s.lock().await;
    };
}

#[derive(Clone)]
pub(crate) struct DiscordIPCSocket {
    readhalf: Arc<Mutex<OwnedReadHalf>>,
    writehalf: Arc<Mutex<OwnedWriteHalf>>,
}

impl DiscordIPCSocket {
    pub(crate) async fn new() -> Result<Self> {
        let path = get_pipe_path();

        if let Some(path) = path {
            let result = UnixStream::connect(&path)
                .await
                .context("Failed to connect to socket; is your app open?");

            if let Ok(stream) = result {
                let (readhalf, writehalf) = stream.into_split();

                return Ok(Self {
                    readhalf: Arc::new(Mutex::new(readhalf)),
                    writehalf: Arc::new(Mutex::new(writehalf)),
                });
            }
        }

        bail!("Socket path doesn't exist, retreating from the losses.")
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

    pub(crate) async fn handle_ipc(&mut self) -> anyhow::Result<()> {
        loop {
            let frame = self.read_frame().await?;
            match frame.opcode {
                3 => {
                    let pack = pack(frame.opcode, frame.body.len() as u32)?;
                    self.write(&pack).await?;
                    self.write(&frame.body).await?;
                }
                2 => break,
                _ => {} // essentially denoting that the client's working
            }
        }

        Ok(())
    }
}
