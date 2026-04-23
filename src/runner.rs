use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};

use anyhow::{Result, anyhow, bail};

use crate::{
    PresenceClient,
    socket::DiscordSock,
    types::{Activity, DynamicRPCFrame, IPCCommand, RPCFrame, ReadyData},
    utils::get_current_timestamp,
};

const MULTIPLE_RUN_CALL_ERR: &str = "PresenceRunner::run() called more than once";

/// A runner that manages the Discord RPC background task.
/// Create a runner, configure it, run it to get a client handle, then clone the handle for sharing.
pub struct PresenceRunner {
    rx: Option<tokio::sync::mpsc::Receiver<IPCCommand>>,
    client: PresenceClient,
    join_handle: Option<JoinHandle<Result<()>>>,
    on_ready: Option<Box<dyn Fn(ReadyData) + Send + Sync + 'static>>,
    do_verbose_errors: bool,
}

impl PresenceRunner {
    #[must_use]
    pub fn new(client_id: &str) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let client = PresenceClient {
            tx,
            client_id: client_id.to_string(),
            running: Arc::new(AtomicBool::new(false)),
        };

        Self {
            rx: Some(rx),
            client,
            join_handle: None,
            on_ready: None,
            do_verbose_errors: false,
        }
    }

    /// Run a particular closure after receiving the READY event from the Discord RPC server.
    ///
    /// This event can fire multiple times depending on how many times the client needs to reconnect with Discord RPC.
    pub fn on_ready<F: Fn(ReadyData) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_ready = Some(Box::new(f));
        self
    }

    /// Enable verbose error logging for RPC and code events.
    pub fn show_errors(mut self) -> Self {
        self.do_verbose_errors = true;
        self
    }

    /// Run the runner.
    /// Must be called before any client handle operations.
    pub async fn run(&mut self, wait_for_ready: bool) -> Result<&PresenceClient> {
        if self.client.running.swap(true, Ordering::SeqCst) {
            bail!(MULTIPLE_RUN_CALL_ERR)
        }

        let client_id = self.client.client_id.clone();
        let running = self.client.running.clone();
        let do_verbose_errors = self.do_verbose_errors.clone();

        let mut rx = self
            .rx
            .take()
            .ok_or_else(|| anyhow!(MULTIPLE_RUN_CALL_ERR))?;

        // oneshot channel to signal when READY is received the first time
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();

        let on_ready = self.on_ready.take();

        let join_handle = tokio::spawn(async move {
            let mut backoff = 1;
            let mut last_activity: Option<Activity> = None;
            let mut ready_tx = Some(ready_tx);

            let mut session_start: Option<u64> = None;

            'outer: while running.load(Ordering::SeqCst) {
                // initial connect
                let mut socket = match DiscordSock::new().await {
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
                            break;
                        }
                    };

                    if frame.opcode != 1 {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_slice::<RPCFrame>(&frame.body) {
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

                        if json.evt.as_deref() == Some("ERROR") && do_verbose_errors {
                            eprintln!("Discord RPC ready event receiver error: {:?}", json.data);
                        }
                    }
                }

                if let Some(activity) = &last_activity {
                    if let Some(t) = session_start {
                        if let Err(e) = socket.send_activity(activity.clone(), t).await {
                            eprintln!("Discord RPC last activity restore error: {e}")
                        }
                    }
                }

                backoff = 1;

                loop {
                    tokio::select! {
                        biased;

                        cmd = rx.recv() => {
                            match cmd {
                                Some(cmd) => {
                                    match cmd {
                                        IPCCommand::SetActivity { activity } => {
                                            let session_start_unpacked = if let Some(s) = session_start {
                                                s
                                            } else {
                                                get_current_timestamp()?
                                            };

                                            let activity = *activity;
                                            last_activity = Some(activity.clone());

                                            if let Err(e) = socket.send_activity(activity, session_start_unpacked).await {
                                                if do_verbose_errors {
                                                    eprintln!("Discord RPC send_activity error: {e}");
                                                }
                                                break;
                                            }
                                        },
                                        IPCCommand::ClearActivity => {
                                            last_activity = None;
                                            session_start = None;

                                            if let Err(e) = socket.clear_activity().await {
                                                if do_verbose_errors {
                                                    eprintln!("Discord RPC clear_activity error: {e}");
                                                }
                                                break;
                                            }
                                        },
                                        IPCCommand::Close => {
                                            let _ = socket.close().await;
                                            running.store(false, Ordering::SeqCst);
                                            break 'outer;
                                        }
                                    }
                                },
                                None => break,
                            }
                        }

                        frame = socket.read_frame() => {
                            match frame {
                                Ok(frame) => {
                                    match frame.opcode {
                                    1 => {
                                        if let Ok(json) = serde_json::from_slice::<DynamicRPCFrame>(&frame.body) {
                                            if json.evt.as_deref() == Some("ERROR") && do_verbose_errors {
                                                eprintln!("Discord RPC DynamicRPCFrame error: {:?}", json.data);
                                            }
                                        } else {
                                            println!("json error")
                                        }
                                    }
                                    2 => break,
                                    3
                                        if let Err(e) = socket.send_frame(3, frame.body).await => {
                                            if do_verbose_errors {
                                                eprintln!("Discord RPC send_frame error: {e}");
                                            }
                                            break;
                                        }
                                    _ => {}
                                }
                                },
                                Err(e) => {
                                    if do_verbose_errors {
                                        eprintln!("Discord RPC generic frame read error: {e}")
                                    }
                                    break;
                                },
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
    #[must_use]
    pub fn clone_handle(&self) -> PresenceClient {
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
