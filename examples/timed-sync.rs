use std::{thread::sleep, time::Duration};

use filthy_rich::DiscordIPCSync;

fn main() {
    let mut client = DiscordIPCSync::new("1463450870480900160").unwrap();

    // first run
    client.run().unwrap();

    client.set_activity("this runs", "for ten seconds").unwrap();
    sleep(Duration::from_secs(5));
    client.set_activity("believe it", "or not").unwrap();
    sleep(Duration::from_secs(5));
}
