use anyhow::Result;
use filthy_rich::PresenceRunner;

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = PresenceRunner::new("1463450870480900160")
        .show_errors()
        .on_disconnect(|reason| {
            eprintln!("discord rpc disconnected: {reason:?}");
        });

    runner.run(false).await?;
    runner.wait().await?;

    Ok(())
}
