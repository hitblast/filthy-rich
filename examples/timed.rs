// SPDX-License-Identifier: MIT

use std::{thread::sleep, time::Duration};

use filthy_rich::ipc::DiscordIPC;

fn main() {
    let mut client = DiscordIPC::new_from("1463450870480900160").unwrap();

    client.run().unwrap();

    client.set_activity("this runs", "for ten seconds").unwrap();
    sleep(Duration::from_secs(5));
    client.set_activity("believe it", "or not").unwrap();
    sleep(Duration::from_secs(5));
}
