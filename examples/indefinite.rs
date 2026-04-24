use anyhow::Result;
use filthy_rich::{
    PresenceRunner,
    types::{Activity, ActivityType, StatusDisplayType},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = PresenceRunner::new("1463450870480900160")
        .on_ready(|data| println!("Connected to user: {}", data.user.username))
        .on_activity_send(|data| {
            // here `data` is a dynamic `serde_json::Value` for convenient access with the Discord schema
            //
            let name = data.get("name").unwrap().to_string();
            let platform = data.get("platform").unwrap().to_string();

            println!("Activity sent to: {name} (running on {platform})")
        })
        .show_errors() // enables verbose error logging
    ;

    let client = runner.run(true).await?;

    let activity = Activity::new()
        .activity_type(ActivityType::Playing)
        .details("epic game")
        .status_display_type(StatusDisplayType::Details)
        .large_image(
            "game_icon",
            Some("Playing a game"),
            Some("https://hitblast.github.io/"),
        )
        .small_image("status", Some("Online"), None::<String>)
        .build();

    client.set_activity(activity).await?;

    // indefinitely block here
    runner.wait().await?;

    Ok(())
}
