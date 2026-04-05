// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Client for communicating with worker nodes.
//!
//! Defines the components for the client used to communicate with worker nodes from the control
//! node. [`Client`] is an actor that runs its own async task and can be created using
//! [`create_client`]. This will create the instance and start its processing loop in a seperate
//! task returning the handle used by the application to interface with the client actor.

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;

use crate::Result;
use crate::error::Error;
use crate::task;

/// Maxium number of messages for a client actor that can be queued.
const CLIENT_CHANNEL_BUFFER_SIZE: usize = 10;

/// Specifies the messages and responses for the client actor.
pub enum ClientMessage {
    /// Initiate a connection to a worker node.
    Connect {
        addr: String,
    }
}

/// Handle used to communicate with the [`Client`] instance.
pub struct ClientHandle {
    /// Transmission end of the channel used to send requests to the server.
    tx: mpsc::Sender<ClientMessage>,
}

impl ClientHandle {
    /// Create a new [`ClientHandle`] instance.
    pub fn new(tx: mpsc::Sender<ClientMessage>) -> Self {
        Self { tx }
    }
}

/// Create a client actor instance, start its processing loop, and return a handle for it.
pub fn create_client() -> ClientHandle {
    // Channel used for the rest of the application to communicate with the server actor.
    let (loc_tx, loc_rx) = mpsc::channel(CLIENT_CHANNEL_BUFFER_SIZE);

    let client = Client::new(loc_tx.clone(), loc_rx);
    task::spawn(run_client(client));

    ClientHandle::new(loc_tx)
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

        let (net_tx, net_rx) = mpsc::channel(CLIENT_CHANNEL_BUFFER_SIZE);
        self.net_tx = Some(net_tx);

        task::spawn(async move {
            match TcpStream::connect(&addr).await {
                Ok(stream) => {
                    process_stream(stream, &addr, handle, net_rx).await;
                },
                Err(error) => {
                    tracing::error!(?error, ?addr, "failed to connect");
                    return;
                },
            };
        });

        Ok(())
    }

    /// Create and return a [`ClientHandle`] instance for communicating with this client instance.
    fn create_handle(&self) -> ClientHandle {
        ClientHandle::new(self.loc_tx.clone())
    }

    /// Processes a request for the server.
    ///
    /// `msg`:  Message containing the request for the client.
    fn proc_msg(&mut self, msg: ClientMessage) -> Result<()> {
        match msg {
            ClientMessage::Connect { addr } => self.connect(&addr),
        }
    }

    /// Get the next request for the client.
    ///
    /// This will return `None` when the message channel is closed and does not contain any queued
    /// messages. If the message queue is empty, but the channel is not closed, this will sleep
    /// until a message is sent or the channel is closed.
    async fn recv_msg(&mut self) -> Option<ClientMessage> {
        self.loc_rx.recv().await
    }
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
    _client: ClientHandle,
    mut net_rx: mpsc::Receiver<ClientMessage>,
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

