// SPDX-License-Identifier: MIT

use filthy_rich::ipc::DiscordIPCSync;

fn main() {
    let mut client = DiscordIPCSync::new("1463450870480900160").unwrap();

    client.run().unwrap();
    client.set_activity("this runs", "forever").unwrap();
    client.wait().unwrap();
}
