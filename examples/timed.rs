use anyhow::Result;
use std::time::Duration;

use filthy_rich::{Activity, DiscordIPC};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    // create activities for later use
    let activity_1 = Activity::new()
        .details("this runs")
        .state("for ten seconds")
        .build();

    let activity_2 = Activity::new()
        .details("believe it")
        .state("or not")
        .build();

    let closing_activity = Activity::new()
        .details("closing presence in...")
        .duration(Duration::from_secs(5))
        .build();

    // first run
    client.run(true).await?;

    client.set_activity(activity_1).await?;
    sleep(Duration::from_secs(5)).await;
    client.set_activity(activity_2).await?;
    sleep(Duration::from_secs(5)).await;
    client.set_activity(closing_activity).await?;
    sleep(Duration::from_secs(5)).await;

    Ok(())
}
