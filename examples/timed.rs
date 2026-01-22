use std::time::Duration;

use anyhow::Result;
use filthy_rich::ipc::DiscordIPC;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new_from("1463450870480900160").await?;

    client.run().await?;

    client.set_activity("this runs", "for ten seconds").await?;
    sleep(Duration::from_secs(5)).await;
    client.set_activity("believe it", "or not").await?;
    sleep(Duration::from_secs(5)).await;

    Ok(())
}
