use anyhow::Result;
use filthy_rich::DiscordIPC;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160").await?;

    let handle = client.run().await?; // spawns IPC loop
    client.set_activity("this runs", "forever").await?; // set activity

    handle.await??;
    Ok(())
}
