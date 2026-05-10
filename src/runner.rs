use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};

use crate::{
    PresenceClient,
    errors::{DisconnectReason, DiscordSockError, PresenceRunnerError},
    socket::{DiscordSock, Opcode},
    types::{
        ActivitySpec, DynamicRPCFrame, IPCCommand, ReadyRPCFrame,
        data::{ActivityResponseData, ReadyData},
    },
    utils::get_current_timestamp,
};

type Callback<T> = Option<Arc<dyn Fn(T) + Send + Sync + 'static>>;

macro_rules! impl_callback {
    ($name:ident, $arg:ty, $doc:expr) => {
        #[doc = $doc]
        pub fn $name<F: Fn($arg) + Send + Sync + 'static>(mut self, f: F) -> Self {
            self.$name = Some(Arc::new(f));
            self
        }
    };
}

/// A runner that manages the Discord RPC background task.
/// Create a runner, configure it, run it to get a client handle, then clone the handle for sharing.
pub struct PresenceRunner {
    rx: Option<tokio::sync::mpsc::Receiver<IPCCommand>>,
    client: PresenceClient,
    join_handle: Option<JoinHandle<()>>,
    on_ready: Callback<ReadyData>,
    on_activity_send: Callback<ActivityResponseData>,
    on_disconnect: Callback<DisconnectReason>,
    on_retry: Callback<usize>,
    show_errors: bool,
    max_retries: usize,
}

impl PresenceRunner {
    #[must_use]
    /// Create a new [`PresenceRunner`] instance. Requires the client ID of your chosen app from the
    /// [Discord Developer Portal](https://discord.com/developers/applications).
    pub fn new(client_id: impl Into<String>) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let client = PresenceClient {
            tx,
            client_id: client_id.into(),
        };

        Self {
            rx: Some(rx),
            client,
            join_handle: None,
            on_ready: None,
            on_activity_send: None,
            on_disconnect: None,
            on_retry: None,
            show_errors: false,
            max_retries: 0,
        }
    }

    impl_callback!(
        on_ready,
        ReadyData,
        "Runs a particular closure after receiving a READY event.

This can fire multiple times depending on how many times the client
needs to disconnect and reconnect."
    );

    impl_callback!(
        on_activity_send,
        ActivityResponseData,
        "Run a particular closure after ensuring that a [`PresenceClient::set_activity`] has successfully managed to pass its data through the IPC channel.

This can fire multiple times."
    );

    impl_callback!(
        on_disconnect,
        DisconnectReason,
        "Runs a particular closure after the RPC connection is lost.

Unlike `on_retry`, this fires only when a previously initialized connection drops, which means that you should prefer
`on_retry` instead if you want more accurate responses to the initial connect or handshake failures, and `on_disconnect`
for reacting to post-connection failures.

This can fire multiple times depending on how many times the client disconnects and reconnects again."
    );

    impl_callback!(
        on_retry,
        usize,
        "Runs a particular closure when retrying for socket creation or handshake.

This can fire multiple times, or for a limited amount depending on the amount of maximum
retries that has been set (through [`PresenceRunner::set_max_retries`]).

The closure parameter is the count of retries done at the time of its execution."
    );

    /// Enable verbose error logging over [`std::io::stderr`] writes for RPC and code events.
    #[must_use]
    pub fn show_errors(mut self) -> Self {
        self.show_errors = true;
        self
    }

    /// Sets the amount of maximum retries to do on socket creation and handshakes before the runner should give up.
    ///
    /// By default this is set to `0` internally, which means the inner loop would retry indefinitely.
    #[must_use]
    pub fn set_max_retries(mut self, count: usize) -> Self {
        self.max_retries = count;
        self
    }

    /// Run the runner.
    /// Must be called before any client handle operations.
    pub async fn run(
        &mut self,
        wait_for_ready: bool,
    ) -> Result<&PresenceClient, PresenceRunnerError> {
        if self.join_handle.is_some() {
            return Err(PresenceRunnerError::MultipleRun);
        }

        let client_id = self.client.client_id.clone();
        let show_errors = self.show_errors;
        let max_retries = self.max_retries;

        let mut rx = self
            .rx
            .take()
            .ok_or_else(|| PresenceRunnerError::ReceiverError)?;

        // oneshot channel to signal when READY is received the first time
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();

        // executable closers (executed within the loop)
        let on_ready = self.on_ready.take();
        let on_activity_send = self.on_activity_send.take();
        let on_disconnect = self.on_disconnect.take();
        let on_retry = self.on_retry.take();

        let join_handle = tokio::spawn(async move {
            const RETRY_DELAY: Duration = Duration::from_secs(1);
            let mut last_activity: Option<ActivitySpec> = None;
            let mut ready_tx = Some(ready_tx);
            let mut retries = 0;

            let mut session_start: Option<u64> = None;

            'outer: loop {
                if max_retries != 0 && retries >= max_retries {
                    break;
                }

                // initial connect
                let mut socket = match DiscordSock::new().await {
                    Ok(s) => s,
                    Err(_) => {
                        sleep(RETRY_DELAY).await;

                        retries += 1;
                        if let Some(f) = &on_retry {
                            let f = f.clone();
                            tokio::spawn(async move { f(retries) });
                        }

                        continue;
                    }
                };

                // initial handshake
                if socket.do_handshake(&client_id).await.is_err() {
                    sleep(RETRY_DELAY).await;

                    retries += 1;
                    if let Some(f) = &on_retry {
                        let f = f.clone();
                        tokio::spawn(async move { f(retries) });
                    }

                    continue;
                }

                // ready loop
                loop {
                    let frame = match socket.read_frame().await {
                        Ok(f) => f,
                        Err(_) => {
                            sleep(RETRY_DELAY).await;
                            retries += 1;
                            if let Some(f) = &on_retry {
                                let f = f.clone();
                                tokio::spawn(async move { f(retries) });
                            }
                            continue 'outer;
                        }
                    };

                    if Opcode::Frame != frame.opcode {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_slice::<ReadyRPCFrame>(&frame.body) {
                        if json.cmd.as_deref() == Some("DISPATCH")
                            && json.evt.as_deref() == Some("READY")
                        {
                            if let Some(tx) = ready_tx.take() {
                                let _ = tx.send(());
                            }
                            if let Some(f) = &on_ready {
                                if let Some(data) = json.data {
                                    let f = f.clone();
                                    tokio::spawn(async move { f(data) });
                                }
                            }

                            // reset retires here
                            retries = 0;
                            break;
                        }

                        if json.evt.as_deref() == Some("ERROR") && show_errors {
                            eprintln!("Discord RPC ready event receiver error: {:?}", json.data);
                        }
                    }
                }

                // restore last activity (if any)
                if let Some(activity) = &last_activity {
                    if let Some(t) = session_start {
                        if let Err(e) = socket.send_activity(activity.clone(), t).await {
                            if show_errors {
                                eprintln!("Discord RPC last activity restore error: {e}")
                            }
                        }
                    }
                }

                // generic loop for receiving commands and responding to pings from Discord itself
                let disconnect_reason = loop {
                    tokio::select! {
                        cmd = rx.recv() => {
                            match cmd {
                                Some(cmd) => {
                                    match cmd {
                                        IPCCommand::SetActivity { activity } => {
                                            let session_start_unpacked = if let Some(s) = session_start {
                                                s
                                            } else {
                                                match get_current_timestamp() {
                                                    Ok(t) => {
                                                        session_start = Some(t);
                                                        t
                                                    },
                                                    Err(e) => {
                                                        if show_errors {
                                                            eprintln!("Discord RPC pre-send_activity time parsing error: {e}")
                                                        }
                                                        break Some(DisconnectReason::OldRelicComputer(e.to_string()));
                                                    }
                                                }
                                            };

                                            let activity = *activity;
                                            last_activity = Some(activity.clone());

                                            if let Err(e) = socket.send_activity(activity, session_start_unpacked).await {
                                                if show_errors {
                                                    eprintln!("Discord RPC send_activity error: {e}");
                                                }
                                                break Some(DisconnectReason::SendActivityError(e.to_string()));
                                            }
                                        },
                                        IPCCommand::ClearActivity => {
                                            last_activity = None;
                                            session_start = None;

                                            if let Err(e) = socket.clear_activity().await {
                                                if show_errors {
                                                    eprintln!("Discord RPC clear_activity error: {e}");
                                                }
                                                break Some(DisconnectReason::ClearActivityError(e.to_string()));
                                            }
                                        },
                                        IPCCommand::Close { done_tx }=> {
                                            let _ = socket.close().await;
                                            let _ = done_tx.send(());
                                            break 'outer;
                                        }
                                    }
                                },
                                None => break Some(DisconnectReason::ClientChannelClosed),
                            }
                        }

                        frame = socket.read_frame() => {
                            match frame {
                                Ok(frame) => {
                                    match frame.opcode {
                                        Opcode::Frame => {
                                            if let Ok(json) = serde_json::from_slice::<DynamicRPCFrame>(&frame.body) {
                                                if json.evt.as_deref() == Some("ERROR") && show_errors {
                                                    eprintln!("Discord RPC DynamicRPCFrame error: {:?}", json.data);
                                                } else if json.cmd.as_deref() == Some("SET_ACTIVITY") {
                                                    if let Some(f) = &on_activity_send {
                                                        if let Some(data) = json.data {
                                                            let data = serde_json::from_value::<ActivityResponseData>(data);

                                                            if let Ok(d) = data {
                                                                let f = f.clone();
                                                                tokio::spawn(async move { f(d)});
                                                            } else if let Err(e) = data{
                                                                println!("{e}")
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        Opcode::Close => break Some(DisconnectReason::ServerClosed),
                                        Opcode::Ping => {
                                            if let Err(e) = socket.send_frame(Opcode::Pong, frame.body).await {
                                                if show_errors {
                                                    eprintln!("Discord RPC send_frame error: {e}");
                                                }
                                                break Some(DisconnectReason::SendFrameError(e.to_string()));
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                Err(e) => {
                                    if show_errors {
                                        eprintln!("Discord RPC generic frame read error: {e}")
                                    }
                                    if let DiscordSockError::IoError(error) = &e {
                                        if error.kind() == std::io::ErrorKind::UnexpectedEof {
                                            break Some(DisconnectReason::PeerClosed);
                                        }
                                    }
                                    break Some(DisconnectReason::ReadFrameError(e.to_string()));
                                },
                            }
                        }
                    }
                };

                if let Some(f) = &on_disconnect {
                    f(disconnect_reason.unwrap_or(DisconnectReason::Unknown));
                }

                sleep(RETRY_DELAY).await;
            }
        });

        self.join_handle = Some(join_handle);

        if wait_for_ready {
            match ready_rx.await {
                Ok(()) => (),
                Err(_) => return Err(PresenceRunnerError::ExitBeforeReady),
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
    ///
    /// NOTE: If there's no `join_handle` present, the function will do nothing and
    /// just return blank.
    pub async fn wait(&mut self) -> Result<(), PresenceRunnerError> {
        if let Some(handle) = self.join_handle.take() {
            handle.await?;
        }

        Ok(())
    }
}
