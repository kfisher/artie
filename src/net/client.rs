// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Client for communicating with worker nodes.
//!
//! Defines the components for the client used to communicate with worker nodes from the control
//! node. [`Client`] is an actor that runs its own async task and can be created using
//! [`create_client`]. This will create the instance and start its processing loop in a seperate
//! task returning the handle used by the application to interface with the client actor.
//!
//! TODO: Update

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;

use crate::Result;
use crate::error::Error;
use crate::task;

/// Maxium number of messages for a client actor that can be queued.
const CLIENT_CHANNEL_BUFFER_SIZE: usize = 10;

/// Initial amount of time to wait before attempting to connect to a worker node after a failed
/// attempt or disconnection.
const BASE_DELAY: Duration = Duration::from_secs(1);

/// The maximum amount of time to wait before attempting to connect to a worker node after a failed
/// attempt or disconnection.
const MAX_DELAY: Duration = Duration::from_secs(60);

/// The number of attempts before the max delay is reached.
///
/// This value is the attempt number where the delay calculation would result in exceeding the
/// maximum delay. `2^6=64` which is greater than the [`MAX_DELAY`] of 60 seconds.
const MAX_BACKOFF_COUNT: u32 = 6;

/// Specifies the messages and responses for the client actor.
pub enum ClientMessage {
    /// Initiate a connection to a worker node.
    Connect {
        addr: String,
    }
}

/// Specifies the messages and responses for the client manager actor.
pub enum ClientManagerMessage {
}

/// Handle used to communicate with the [`Client`] instance.
#[derive(Clone)]
pub struct ClientHandle {
    /// Transmission end of the channel used to send requests to the client actor.
    tx: mpsc::Sender<ClientMessage>,
}

impl ClientHandle {
    /// Create a new [`ClientHandle`] instance.
    pub fn new(tx: mpsc::Sender<ClientMessage>) -> Self {
        Self { tx }
    }
}

/// Handle used to communicate with the [`ClientManager`] instance.
#[derive(Clone)]
pub struct ClientManagerHandle {
    /// Transmission end of the channel used to send requests to the client manager actor.
    tx: mpsc::Sender<ClientManagerMessage>,
}

impl ClientManagerHandle  {
    /// Create a new [`ClientManagerHandle`] instance.
    pub fn new(tx: mpsc::Sender<ClientManagerMessage>) -> Self {
        Self { tx }
    }
}

/// Create a [`ClientManager`] instance, start its processing loop, and return a handle for it.
pub fn create_client_manager() -> ClientManagerHandle {
    // Channel used for the rest of the application to communicate with the client manager actor.
    let (tx, rx) = mpsc::channel(CLIENT_CHANNEL_BUFFER_SIZE);

    let mgr = ClientManager::new(tx.clone(), rx);
    task::spawn(run_client_manager(mgr));

    ClientManagerHandle::new(tx)
}

/// Actor which handles the client side of network communication.
///
/// There will be one client actor instance per configured worker node.
struct Client {
    /// Transmission end of the channel used to send requests to the client instance.
    ///
    /// This isn't used directly by the client. It serves as a "prototype" and cloned when creating
    /// new handles.
    loc_tx: mpsc::Sender<ClientMessage>,

    /// Receiving end of the channel used to send requests to the client.
    ///
    /// This channel is used by the rest of the application to communicate with the client actor.
    loc_rx: mpsc::Receiver<ClientMessage>,

    /// Transmission end of the channel used to send messages to the connected worker.
    ///
    /// `None` when not connected to the worker.
    net_tx: Option<mpsc::Sender<ClientMessage>>,
}

impl Client {
    /// Create a [`Client`] instance.
    ///
    /// `loc_tx`:  Transmission end of the channel used to send requests to the client instance.
    ///
    /// `loc_rx`:  Receiving end of the channel used to send requests to the client.
    fn new(
        loc_tx: mpsc::Sender<ClientMessage>,
        loc_rx: mpsc::Receiver<ClientMessage>,
    ) -> Self {
        Self { loc_tx, loc_rx, net_tx: None }
    }

    /// Connect to the worker node.
    ///
    /// `addr`:  The address of the node to connect to.
    fn connect(&mut self, addr: &str) -> Result<()> {
        let addr = addr.to_owned();

        let handle = self.create_handle();

        let (net_tx, mut net_rx) = mpsc::channel(CLIENT_CHANNEL_BUFFER_SIZE);
        self.net_tx = Some(net_tx);

        task::spawn(async move {
            let mut attempt: u32 = 0;

            loop {
                match TcpStream::connect(&addr).await {
                    Ok(stream) => {
                        attempt = 0;
                        process_stream(stream, &addr, &handle, &mut net_rx).await;
                        tracing::warn!(?addr, "connection lost, will attempt to reconnect");
                    }
                    Err(error) => {
                        tracing::error!(?error, ?addr, attempt, "failed to connect");
                    }
                }

                attempt = attempt.saturating_add(1);
                let delay = if attempt >= MAX_BACKOFF_COUNT {
                    MAX_DELAY 
                } else {
                    BASE_DELAY * 2u32.saturating_pow(attempt)
                };

                tracing::info!(?addr, attempt, ?delay, "reconnecting after delay");
                tokio::time::sleep(delay).await;
            }
        });

        Ok(())
    }

    /// Create and return a [`ClientHandle`] instance for communicating with this client instance.
    fn create_handle(&self) -> ClientHandle {
        ClientHandle::new(self.loc_tx.clone())
    }

    /// Processes a request for the client actor.
    ///
    /// `msg`:  Message containing the request for the client.
    fn proc_msg(&mut self, msg: ClientMessage) -> Result<()> {
        match msg {
            ClientMessage::Connect { addr } => self.connect(&addr),
        }
    }

    /// Get the next request for the client actor.
    ///
    /// This will return `None` when the message channel is closed and does not contain any queued
    /// messages. If the message queue is empty, but the channel is not closed, this will sleep
    /// until a message is sent or the channel is closed.
    async fn recv_msg(&mut self) -> Option<ClientMessage> {
        self.loc_rx.recv().await
    }
}

/// Actor which manages client instances.
struct ClientManager {
    /// Transmission end of the channel used to send requests to the client manager actor instance.
    ///
    /// This isn't used directly. It serves as a "prototype" and cloned when creating new handles.
    tx: mpsc::Sender<ClientManagerMessage>,

    /// Receiving end of the channel used to send requests to the client manager actor instance.
    ///
    /// This channel is used by the rest of the application to communicate with the actor.
    rx: mpsc::Receiver<ClientManagerMessage>,
}

impl ClientManager {
    /// Create a [`ClientManager`] instance.
    ///
    /// `tx`:  Transmission end of the channel used to send requests to the client instance.
    ///
    /// `rx`:  Receiving end of the channel used to send requests to the client.
    fn new(
        tx: mpsc::Sender<ClientManagerMessage>,
        rx: mpsc::Receiver<ClientManagerMessage>,
    ) -> Self {
        Self { tx, rx }
    }

    /// Processes a request for the client manager actor.
    ///
    /// `msg`:  Message containing the request.
    fn proc_msg(&mut self, msg: ClientManagerMessage) -> Result<()> {
        match msg {
        }
    }

    /// Get the next request for the client manager actor.
    ///
    /// This will return `None` when the message channel is closed and does not contain any queued
    /// messages. If the message queue is empty, but the channel is not closed, this will sleep
    /// until a message is sent or the channel is closed.
    async fn recv_msg(&mut self) -> Option<ClientManagerMessage> {
        self.rx.recv().await
    }
}

/// Create a client actor instance, start its processing loop, and return a handle for it.
fn create_client() -> ClientHandle {
    // Channel used for the rest of the application to communicate with the client actor.
    let (loc_tx, loc_rx) = mpsc::channel(CLIENT_CHANNEL_BUFFER_SIZE);

    let client = Client::new(loc_tx.clone(), loc_rx);
    task::spawn(run_client(client));

    ClientHandle::new(loc_tx)
}

/// Process communication with a connected worker node.
///
/// This will run until the worker drops the connection or the client actor closes the channel it
/// uses to send messages.
///
/// `stream`:  The stream for the connected worker.
///
/// `addr`:  The address of the connected worker.
///
/// `client`:  Handle to the client actor. Used to forward messages from the connected worker node
/// to the actor for processing.
///
/// `net_rx`:  Receiving end of the channel used by the server actor to send messages to the
/// connected worker node.
async fn process_stream(
    stream: TcpStream,
    addr: &str,
    _client: &ClientHandle,
    net_rx: &mut mpsc::Receiver<ClientMessage>,
) {
    let (reader, mut writer) = stream.into_split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        tracing::info!(?addr, "connection close by remote");
                        break;
                    },
                    Ok(_) => {
                        // TODO: Parse message and then relay it to the server via the handle.
                        print!("Received: {line}");
                        line.clear();
                    },
                    Err(error) => {
                        tracing::error!(?addr, ?error, "failed to read message");
                        break;
                    },
                }
            }
            msg = net_rx.recv() => {
                match msg {
                    Some(_) => {
                        // TODO: Serialize the message and send to the client.
                        if let Err(error) = writer.write_all("TODO".as_bytes()).await {
                            tracing::error!(?addr, ?error, "failed to send message");
                            break;
                        }
                    },
                    None => {
                        tracing::info!("client channel closed. exiting");
                        break;
                    }
                }
            }

        }
    }
}

/// Runs the client processing loop.
///
/// This will process requests from the application for the worker node. It will run until the
/// message channel is closed.
async fn run_client(mut client: Client) {
    tracing::info!("client started");

    while let Some(msg) = client.recv_msg().await {
        if let Err(error) = client.proc_msg(msg) {
            tracing::error!(?error, "failed to process client request");
        }
    }

    tracing::info!("client exited");
}

/// Runs the client manager processing loop.
///
/// This will process requests from the application for the client manager. It will run until the
/// message channel is closed.
async fn run_client_manager(mut mgr: ClientManager) {
    tracing::info!("client manager started");

    while let Some(msg) = mgr.recv_msg().await {
        if let Err(error) = mgr.proc_msg(msg) {
            tracing::error!(?error, "failed to process client manager request");
        }
    }

    tracing::info!("client manager exited");
}

