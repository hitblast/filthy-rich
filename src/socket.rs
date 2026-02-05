// SPDX-License-Identifier: MIT

use anyhow::{Context, Result, bail};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    sync::{Arc, Mutex},
};

use crate::utils::{get_pipe_path, pack};

pub(crate) struct Frame {
    pub opcode: u32,
    pub body: Vec<u8>,
}

macro_rules! acquire {
    ($s:expr, $x:ident) => {
        let mut $x = $s
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?;
    };
}

#[derive(Clone)]
pub(crate) struct DiscordIPCSocket {
    readhalf: Arc<Mutex<UnixStream>>,
    writehalf: Arc<Mutex<UnixStream>>,
}

impl DiscordIPCSocket {
    pub(crate) fn new() -> Result<Self> {
        let path = get_pipe_path();

        if let Some(path) = path {
            let result = UnixStream::connect(&path)
                .context("Failed to connect to socket; is your app open?");

            if let Ok(writehalf) = result {
                let readhalf = writehalf
                    .try_clone()
                    .context("Failed to clone stream for read thread.")?;

                return Ok(Self {
                    readhalf: Arc::new(Mutex::new(readhalf)),
                    writehalf: Arc::new(Mutex::new(writehalf)),
                });
            }
        }

        bail!("Socket path doesn't exist, retreating from the losses.")
    }

    pub(crate) fn read(&mut self, buffer: &mut [u8]) -> Result<()> {
        acquire!(&self.readhalf, stream);
        stream.read_exact(buffer)?;
        Ok(())
    }

    pub(crate) fn write(&mut self, buffer: &[u8]) -> Result<()> {
        acquire!(&self.writehalf, stream);
        stream.write_all(buffer)?;
        Ok(())
    }

    pub(crate) fn read_frame(&mut self) -> Result<Frame> {
        let mut header = [0u8; 8];
        self.read(&mut header)?;

        let opcode = u32::from_le_bytes(header[0..4].try_into()?);
        let len = u32::from_le_bytes(header[4..8].try_into()?) as usize;

        let mut body = vec![0u8; len];
        self.read(&mut body)?;

        Ok(Frame { opcode, body })
    }

    pub(crate) fn handle_ipc(&mut self) -> anyhow::Result<()> {
        loop {
            let frame = self.read_frame()?;
            match frame.opcode {
                3 => {
                    let pack = pack(frame.opcode, frame.body.len() as u32)?;
                    self.write(&pack)?;
                    self.write(&frame.body)?;
                }
                2 => break,
                _ => {} // essentially denoting that the client's working
            }
        }

        Ok(())
    }
}
