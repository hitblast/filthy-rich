use anyhow::Result;
use filthy_rich::{Activity, DiscordIPC};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    client.run(true).await?;

    let activity = Activity::new()
        .details("this runs forever")
        .large_image("game_icon", Some("Playing a game"))
        .small_image("status", Some("Online"))
        .build();

    client.set_activity(activity).await?;
    client.wait().await?;

    Ok(())
}
