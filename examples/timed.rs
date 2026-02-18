use anyhow::Result;
use std::time::Duration;

use filthy_rich::DiscordIPC;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160");

    // first run
    client.run().await.unwrap();

    client.set_activity("this runs", "for ten seconds").await?;
    sleep(Duration::from_secs(5)).await;
    client.set_activity("believe it", "or not").await?;
    sleep(Duration::from_secs(5)).await;

    Ok(())
}
