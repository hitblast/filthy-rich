use anyhow::Result;
use filthy_rich::{Activity, DiscordIPCRunner};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = DiscordIPCRunner::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    let client = runner.run(true).await?;

    let activity = Activity::new()
        .details("this runs forever")
        .large_image("game_icon", Some("Playing a game"))
        .small_image("status", Some("Online"))
        .build();

    client.set_activity(activity).await?;
    runner.wait().await?;

    Ok(())
}
