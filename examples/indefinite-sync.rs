use filthy_rich::{Activity, DiscordIPCSync};

fn main() {
    let mut client = DiscordIPCSync::new("1463450870480900160")
        .unwrap()
        .on_ready(|data| println!("Connected to user: {}", data.user.username));

    client.run(true).unwrap();

    let activity = Activity::new("this runs forever");

    client.set_activity(activity).unwrap();
    client.wait().unwrap();
}
