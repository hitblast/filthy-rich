// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::{
    collections::HashSet,
    env::var,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[must_use]
pub(crate) fn get_pipe_path() -> Option<PathBuf> {
    let mut candidates = HashSet::new();

    #[cfg(target_os = "windows")]
    possible_paths.insert(r"\\?\pipe\discord-ipc-".to_string());

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

pub(crate) fn get_current_timestamp() -> Result<u64> {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(ts)
}

pub(crate) fn pack(opcode: u32, data_len: u32) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();

    for byte_array in &[opcode.to_le_bytes(), data_len.to_le_bytes()] {
        bytes.extend_from_slice(byte_array);
    }

    Ok(bytes)
}
