use anyhow::Result;
use std::time::Duration;

use filthy_rich::DiscordIPC;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160")
        .on_ready(|| println!("filthy-rich is READY to set activities."));

    // first run
    client.run(true).await?;

    client
        .set_activity("this runs".to_string(), Some("for ten seconds".to_string()))
        .await?;
    sleep(Duration::from_secs(5)).await;
    client
        .set_activity("believe it".to_string(), Some("or not!".to_string()))
        .await?;
    sleep(Duration::from_secs(5)).await;

    Ok(())
}
