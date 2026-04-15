use anyhow::Result;
use filthy_rich::{PresenceRunner, types::Activity};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = PresenceRunner::new("1463450870480900160");

    let activity = Activity::new()
        .name("test")
        .details("Some thing?")
        .state("bad")
        .build();

    let client = runner.run(true).await?;
    client.set_activity(activity).await?;

    // indefinitely block here
    runner.wait().await?;

    Ok(())
}
