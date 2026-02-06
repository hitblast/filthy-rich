use std::time::Duration;

use filthy_rich::ipc::DiscordIPC;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let mut client = DiscordIPC::new("1463450870480900160").await.unwrap();

    client.run().await.unwrap();

    client
        .set_activity("this runs", "for ten seconds")
        .await
        .unwrap();
    sleep(Duration::from_secs(5)).await;
    client.set_activity("believe it", "or not").await.unwrap();
    sleep(Duration::from_secs(5)).await;
}
