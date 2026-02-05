// SPDX-License-Identifier: MIT

use filthy_rich::ipc::DiscordIPC;

fn main() {
    let mut client = DiscordIPC::new_from("1463450870480900160").unwrap();

    client.run().unwrap();
    client.set_activity("this runs", "forever").unwrap();
    client.wait().unwrap();
}
