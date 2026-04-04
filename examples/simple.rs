use anyhow::Result;
use filthy_rich::{Activity, DiscordIPCRunner};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = DiscordIPCRunner::new("1463450870480900160");

    let activity = Activity::build_empty();

    let client = runner.run(true).await?;
    client.set_activity(activity).await?;
    runner.wait().await?;

    Ok(())
}
