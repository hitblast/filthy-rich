use anyhow::Result;
use filthy_rich::{
    PresenceRunner,
    types::{Activity, ActivityType, StatusDisplayType},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = PresenceRunner::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    let client = runner.run(true).await?;

    let activity = Activity::new()
        .activity_type(ActivityType::Competing)
        .details("epic game")
        .status_display_type(StatusDisplayType::Details)
        .large_image("game_icon", Some("Playing a game"))
        .small_image("status", Some("Online"))
        .build();

    client.set_activity(activity).await?;

    // indefinitely block here
    runner.wait().await?;

    Ok(())
}
