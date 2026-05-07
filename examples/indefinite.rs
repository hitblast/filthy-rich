use filthy_rich::{
    PresenceRunner,
    errors::PresenceError,
    types::{Activity, ActivityType, StatusDisplayType},
};

#[tokio::main]
async fn main() -> Result<(), PresenceError> {
    let mut runner = PresenceRunner::new("1463450870480900160")
        .on_ready(|data| {
            println!(
                "RPC version: v{}; Connected to user: {}",
                data.version(),
                data.user().username(),
            )
        })
        .on_activity_send(|data| {
            println!(
                "Activity sent to app! (running on {})\nCreated at: {}",
                data.platform().unwrap_or_default(),
                data.activity().created_at()
            );
        })
        .on_disconnect(|f| println!("Disconnected: {f:?}"))
        .show_errors() // enables verbose error logging
        .on_retry(move |c| {
            if c % 10 == 0 {
                println!("Retry count {c}; is Discord open?");
            }
        });

    let client = runner.run(true).await?;

    // the activity can include any combination of builder function calls
    let activity = Activity::new()
        .activity_type(ActivityType::Playing)
        .details("epic game")
        .details_url("https://github.com/hitblast")
        .status_display_type(StatusDisplayType::Details)
        .large_image("game_icon")
        .large_text("Playing a game")
        .large_url("https://hitblast.github.io/")
        .small_image("status")
        .small_text("Online")
        .build()?;

    client.set_activity(activity).await?;

    // indefinitely block here
    runner.wait().await?;

    Ok(())
}
