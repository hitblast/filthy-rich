use anyhow::Result;
use filthy_rich::{Activity, DiscordIPC};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    client.run(true).await?;

    let activity = Activity::new("this runs forever");

    client.set_activity(activity).await?;
    client.wait().await?;

    Ok(())
}
