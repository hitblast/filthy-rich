use anyhow::Result;
use filthy_rich::DiscordIPC;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160");

    println!("Performing as client: {}", client.client_id());

    client.run().await?;
    client.set_activity("this runs", "forever").await?;
    client.wait().await?;

    Ok(())
}
