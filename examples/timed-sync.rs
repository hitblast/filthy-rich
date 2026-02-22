use std::{thread::sleep, time::Duration};

use filthy_rich::{Activity, DiscordIPCSync};

fn main() {
    let mut client = DiscordIPCSync::new("1463450870480900160")
        .unwrap()
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    // create activities for later use
    let activity_1 = Activity::new("this runs").state("for ten seconds");
    let activity_2 = Activity::new("believe it").state("or not");
    let close_activity = Activity::new("closing presence in...").duration(Duration::from_secs(5));

    // first run
    client.run(true).unwrap();

    client.set_activity(activity_1).unwrap();
    sleep(Duration::from_secs(5));
    client.set_activity(activity_2).unwrap();
    sleep(Duration::from_secs(5));
    client.set_activity(close_activity).unwrap();
    sleep(Duration::from_secs(5));
}
