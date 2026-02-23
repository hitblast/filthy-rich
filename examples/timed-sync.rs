use std::{thread::sleep, time::Duration};

use filthy_rich::{Activity, DiscordIPCSync};

fn main() {
    let mut client = DiscordIPCSync::new("1463450870480900160")
        .unwrap()
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
    let close_activity = Activity::new()
        .details("closing presence in...")
        .duration(Duration::from_secs(5))
        .build();

    // first run
    client.run(true).unwrap();

    client.set_activity(activity_1).unwrap();
    sleep(Duration::from_secs(5));
    client.set_activity(activity_2).unwrap();
    sleep(Duration::from_secs(5));
    client.set_activity(close_activity).unwrap();
    sleep(Duration::from_secs(5));
}
