// SPDX-License-Identifier: MIT

use anyhow::Result;
use filthy_rich::ipc::DiscordIPC;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new_from("1463450870480900160").await?;

    client.run().await?;
    client.set_activity("this runs", "forever").await?;
    client.wait().await?;

    Ok(())
}
