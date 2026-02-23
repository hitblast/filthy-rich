use anyhow::Result;
use filthy_rich::{Activity, DiscordIPC};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160");

    let activity = Activity::build_empty();

    client.run(true).await?;
    client.set_activity(activity).await?;
    client.wait().await?;

    Ok(())
}
