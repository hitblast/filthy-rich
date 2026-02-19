use anyhow::Result;
use filthy_rich::DiscordIPC;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160")
        .on_ready(|| println!("filthy-rich is READY to set activities."));

    client.run(true).await?;

    client
        .set_activity("this runs forever".to_string(), None)
        .await?;
    client.wait().await?;

    Ok(())
}
