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

    client.clear_activity().unwrap();

    // if you want to drop the connection here:
    // client.close().unwrap();

    // optional sleep
    sleep(Duration::from_secs(2));

    // 2nd run
    client.set_activity("this is the", "second run").unwrap();
    sleep(Duration::from_secs(5));
    client
        .set_activity("which also runs", "for ten seconds")
        .unwrap();
    sleep(Duration::from_secs(5));
}
