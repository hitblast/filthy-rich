use anyhow::Result;
use filthy_rich::{DiscordIPCRunner, types::Activity};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = DiscordIPCRunner::new("1463450870480900160");

    let activity = Activity::new().details("Some thing?").state("bad").build();

    let client = runner.run(true).await?;
    client.set_activity(activity).await?;
    runner.wait().await?;

    Ok(())
}
