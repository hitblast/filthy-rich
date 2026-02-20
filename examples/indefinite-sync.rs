use filthy_rich::DiscordIPCSync;

fn main() {
    let mut client = DiscordIPCSync::new("1463450870480900160")
        .unwrap()
        .on_ready(|| println!("filthy-rich is READY to set activity updates."));

    client.run(true).unwrap();

    client
        .set_activity("this runs forever".to_string(), None)
        .unwrap();
    client.wait().unwrap();
}
