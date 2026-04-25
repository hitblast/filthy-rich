use anyhow::Result;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use filthy_rich::{PresenceRunner, types::Activity};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // simple atomic counter
    let count = Arc::new(AtomicU8::new(0));

    let mut runner = PresenceRunner::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username))
        .on_activity_send(move |_| {

            // increments the counter with every send_activity()
            let val = count.fetch_add(1, Ordering::Relaxed) + 1;

            println!("Activity {val} sent successfully.")

        })
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
        .small_image("status")
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
