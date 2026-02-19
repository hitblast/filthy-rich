use std::{thread::sleep, time::Duration};

use filthy_rich::DiscordIPCSync;

fn main() {
    let mut client = DiscordIPCSync::new("1463450870480900160")
        .unwrap()
        .on_ready(|| println!("filthy-rich is READY to set activity updates."));

    // first run
    client.run(true).unwrap();

    client
        .set_activity("this runs".to_string(), Some("for ten seconds".to_string()))
        .unwrap();
    sleep(Duration::from_secs(5));
    client
        .set_activity("believe it".to_string(), Some("or not!".to_string()))
        .unwrap();
    sleep(Duration::from_secs(5));
}
