use std::time::Duration;

use anyhow::Result;
use filthy_rich::{
    DiscordIPCRunner,
    types::{Activity, ActivityType, StatusDisplayType},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = DiscordIPCRunner::new("1463450870480900160");

    let activity = Activity::new_with_type(ActivityType::Listening)
        .details("Things")
        .duration(Duration::from_secs(20))
        .status_display_type(StatusDisplayType::Details)
        .build();

    let client = runner.run(true).await?;
    client.set_activity(activity).await?;
    runner.wait().await?;

    Ok(())
}
