use anyhow::Result;
use std::time::Duration;

use filthy_rich::{PresenceRunner, types::Activity};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = PresenceRunner::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username))
        .show_errors() // enables verbose error logging
    ;

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
    let client = runner.run(true).await?;

    client.set_activity(activity_1).await?;
    sleep(Duration::from_secs(5)).await;
    client.set_activity(activity_2).await?;
    sleep(Duration::from_secs(5)).await;
    client.set_activity(closing_activity).await?;
    sleep(Duration::from_secs(5)).await;

    Ok(())
}
