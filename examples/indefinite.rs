use filthy_rich::ipc::DiscordIPC;

#[tokio::main]
async fn main() {
    let mut client = DiscordIPC::new("1463450870480900160").await.unwrap();

    client.run().await.unwrap();
    client.set_activity("this runs", "forever").await.unwrap();
    client.wait().await.unwrap();
}
