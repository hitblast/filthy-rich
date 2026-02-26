use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::sleep,
};

use anyhow::{Result, bail};

use crate::{
    socket::DiscordIPCSocket,
    types::{IPCCommand, ReadyData, RpcFrame},
    Activity, DiscordIPCClient,
};

fn get_current_timestamp() -> Result<u64> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs())
}

/// A runner that manages the Discord IPC background task.
/// Create a runner, configure it, run it to get a client handle, then clone the handle for sharing.
pub struct DiscordIPCRunner {
    rx: Option<tokio::sync::mpsc::Receiver<IPCCommand>>,
    client: DiscordIPCClient,
    join_handle: Option<JoinHandle<Result<()>>>,
    on_ready: Option<Box<dyn Fn(ReadyData) + Send + Sync + 'static>>,
}

impl DiscordIPCRunner {
    pub fn new(client_id: &str) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let client = DiscordIPCClient {
            tx,
            client_id: client_id.to_string(),
            running: Arc::new(AtomicBool::new(false)),
        };
        Self {
            rx: Some(rx),
            client,
            join_handle: None,
            on_ready: None,
        }
    }

    /// Run a particular closure after receiving the READY event from the local Discord IPC server.
    pub fn on_ready<F: Fn(ReadyData) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
        self
    }

    /// Run the runner.
    /// Must be called before any client handle operations.
    pub async fn run(&mut self, wait_for_ready: bool) -> Result<&DiscordIPCClient> {
        if self.client.running.swap(true, Ordering::SeqCst) {
            bail!(
                "Cannot run multiple instances of .run() for DiscordIPCRunner, or when a session is still closing."
            )
        }

        let client_id = self.client.client_id.clone();
        let running = self.client.running.clone();
        let mut rx = self.rx.take().unwrap();

        // oneshot channel to signal when READY is received the first time
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();

        let on_ready = self.on_ready.take();

        let join_handle = tokio::spawn(async move {
            let mut backoff = 1;
            let mut last_activity: Option<Activity> = None;
            let mut ready_tx = Some(ready_tx);
            let mut should_continue = true;

            while should_continue && running.load(Ordering::SeqCst) {
                // initial connect
                let mut socket = match DiscordIPCSocket::new().await {
                    Ok(s) => s,
                    Err(_) => {
                        sleep(Duration::from_secs(backoff)).await;
                        continue;
                    }
                };

                // initial handshake
                if socket.do_handshake(&client_id).await.is_err() {
                    sleep(Duration::from_secs(backoff)).await;
                    continue;
                }

                loop {
                    let frame = match socket.read_frame().await {
                        Ok(f) => f,
                        Err(_) => {
                            should_continue = true;
                            break;
                        }
                    };

                    if frame.opcode != 1 {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_slice::<RpcFrame>(&frame.body) {
                        if json.cmd.as_deref() == Some("DISPATCH")
                            && json.evt.as_deref() == Some("READY")
                        {
                            if let Some(tx) = ready_tx.take() {
                                let _ = tx.send(());
                            }
                            if let Some(f) = &on_ready {
                                if let Some(data) = json.data {
                                    f(data);
                                }
                            }
                            break;
                        }

                        if json.evt.as_deref() == Some("ERROR") {
                            eprintln!("Discord RPC error: {:?}", json.data);
                        }
                    }
                }

                let session_start = get_current_timestamp()?;

                if let Some(activity) = &last_activity {
                    let _ = socket.send_activity(activity.clone(), session_start).await;
                }

                backoff = 1;

                loop {
                    tokio::select! {
                        Some(cmd) = rx.recv() => {
                            match cmd {
                                IPCCommand::SetActivity { activity } => {
                                    last_activity = Some(activity.clone());

                                    if socket.send_activity(activity, session_start).await.is_err() {
                                        break;
                                    }
                                },
                                IPCCommand::ClearActivity => {
                                    last_activity = None;

                                    if socket.clear_activity().await.is_err() { break; }
                                },
                                IPCCommand::Close => {
                                    let _ = socket.close().await;
                                    running.store(false, Ordering::SeqCst);
                                    should_continue = false;
                                    break;
                                }
                            }
                        }

                        frame = socket.read_frame() => {
                            match frame {
                                Ok(frame) => match frame.opcode {
                                    1 => {
                                        if let Ok(json) = serde_json::from_slice::<RpcFrame>(&frame.body) {
                                            if json.evt.as_deref() == Some("ERROR") {
                                                eprintln!("Discord RPC error: {:?}", json.data);
                                            }
                                        }
                                    }
                                    2 => break,
                                    3 => {
                                        if socket.send_frame(3, frame.body).await.is_err() { break; }
                                    }
                                    _ => {}
                                },
                                Err(_) => break,
                            }
                        }
                    }
                }

                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(4);
            }

            Ok(())
        });

        self.join_handle = Some(join_handle);

        if wait_for_ready {
            match ready_rx.await {
                Ok(()) => (),
                Err(_) => bail!("Background task exited before READY."),
            }
        }

        Ok(&self.client)
    }

    /// Returns a clone of the client handle for sharing.
    pub fn clone_handle(&self) -> DiscordIPCClient {
        self.client.clone()
    }

    /// Waits for the IPC task to finish.
    pub async fn wait(&mut self) -> Result<()> {
        if let Some(handle) = self.join_handle.take() {
            handle.await??;
        }

        Ok(())
    }
}