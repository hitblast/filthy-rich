use anyhow::Result;
use filthy_rich::DiscordIPC;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160").await?;

    println!("Performing as client: {}", client.client_id());

    let handle = client.run().await?; // spawns IPC loop; this enforces tokio workflow
    client.set_activity("this runs", "forever").await?; // set activity
    handle.await??;

    Ok(())
}
