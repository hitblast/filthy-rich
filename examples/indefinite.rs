use anyhow::Result;
use filthy_rich::{
    DiscordIPCRunner,
    types::{Activity, ActivityType, StatusDisplayType},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = DiscordIPCRunner::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    let client = runner.run(true).await?;

    let activity = Activity::new_with_type(ActivityType::Competing)
        .details("epic game")
        .status_display_type(StatusDisplayType::Details)?
        .large_image("game_icon", Some("Playing a game"))
        .small_image("status", Some("Online"))
        .build();

    client.set_activity(activity).await?;
    runner.wait().await?;

    Ok(())
}
