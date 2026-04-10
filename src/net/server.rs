// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Server for the worker node.
//!
//! Defines the components for the server on worker nodes which listens for the connection from the
//! control node.
//!
//! [`Server`] is an actor that runs in its own async task which can be created and started using
//! [`create_and_run_server`]. This will return an instance to [`ServerHandle`] which can be used
//! to handle communication with the actor. 

use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::Result;
use crate::task;

/// Maxium number of messages for a server actor that can be queued.
const SERVER_CHANNEL_BUFFER_SIZE: usize = 10;

/// Specifies the messages and responses for the server actor.
pub enum ServerMessage {
}

/// Handle used to communicate with the [`Server`] instance.
pub struct ServerHandle {
    /// Transmission end of the channel used to send requests to the server.
    tx: mpsc::Sender<ServerMessage>,
}

impl ServerHandle {
    /// Create a new [`ServerHandle`] instance.
    pub fn new(tx: mpsc::Sender<ServerMessage>) -> Self {
        Self { tx }
    }
}

/// Create the [`Server`] instance, start its processing task, and return a handle for it.
pub fn create_and_run_server() -> ServerHandle {
    // Channel used for the rest of the application to communicate with the server actor.
    let (loc_tx, loc_rx) = mpsc::channel(SERVER_CHANNEL_BUFFER_SIZE);

    // Channel used for the server actor to send messages over the network.
    let (net_tx, net_rx) = mpsc::channel(SERVER_CHANNEL_BUFFER_SIZE);

    let server = Server::new(loc_tx.clone(), loc_rx, net_tx);
    task::spawn(run_server(server, net_rx));

    ServerHandle::new(loc_tx)
}

/// Actor which handles the worker side of network communication.
struct Server {
    /// Transmission end of the channel used to send requests to the server instance.
    ///
    /// This isn't used directly by the server. It serves as a "prototype" and cloned when creating
    /// new handles for the server.
    loc_tx: mpsc::Sender<ServerMessage>,

    /// Receiving end of the channel used to send requests to the server.
    ///
    /// This channel is used by the rest of the application to communicate with the server actor.
    loc_rx: mpsc::Receiver<ServerMessage>,

    /// Transmission end of the channel used to send messages to the connected client.
    net_tx: mpsc::Sender<ServerMessage>,
}

impl Server {
    /// Create a [`Server`] instance.
    ///
    /// `loc_tx`:  Transmission end of the channel used to send requests to the server instance.
    ///
    /// `loc_rx`:  Receiving end of the channel used to send requests to the server.
    ///
    /// `net_tx`:  Transmission end of the channel used to send messages to the connected client.
    fn new(
        loc_tx: mpsc::Sender<ServerMessage>,
        loc_rx: mpsc::Receiver<ServerMessage>,
        net_tx: mpsc::Sender<ServerMessage>,
    ) -> Self {
        Self { loc_tx, loc_rx, net_tx }
    }

    /// Create and return a [`ServerHandle`] instance for communicating with this server instance.
    fn create_handle(&self) -> ServerHandle {
        ServerHandle::new(self.loc_tx.clone())
    }

    /// Processes a request for the server.
    ///
    /// `msg`:  Message containing the request for the server.
    fn proc_msg(&mut self, _msg: ServerMessage) -> Result<()> {
        todo!()
    }

    /// Get the next request for the server.
    ///
    /// This will return `None` when the message channel is closed and does not contain any queued
    /// messages. If the message queue is empty, but the channel is not closed, this will sleep
    /// until a message is sent or the channel is closed.
    async fn recv_msg(&mut self) -> Option<ServerMessage> {
        self.loc_rx.recv().await
    }
}

/// Listen for connections from the control node.
///
/// **Note** This will run indefinately until the application exits. Its expected to be run as a 
/// seperate async task.
///
/// `addr`:  The address and port to listen on.
///
/// `server`:  Handle for the server instance.
///
/// `net_rx`:  Receiving end of the channel used by the server actor to send messages to the
/// connected client.
async fn listen(addr: &str, server: ServerHandle, mut net_rx: mpsc::Receiver<ServerMessage>) {
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(?error, ?addr, "failed to listen for connections");
            return;
        }
    };

    tracing::info!(?addr, "waiting for connection");
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                // We only support a single connected client (i.e. the control node) at a time.
                tracing::info!(?peer_addr, "client connected");
                process_stream(stream, peer_addr, &server, &mut net_rx).await;
                tracing::info!("client disconnected");
            },
            Err(error) => {
                tracing::error!(?error, ?addr, "failed to accept connection");
            }
        }
    }
}

/// Process communication with a connected client.
///
/// This will run until the connect client drops the connection or the server actor closes the
/// channel it uses to send messages.
///
/// `stream`:  The stream for the connected client.
///
/// `peer_addr`:  The address of the connected client.
///
/// `server`:  Handle to the server actor. Used to forward messages from the connected client to
/// the actor for processing.
///
/// `net_rx`:  Receiving end of the channel used by the server actor to send messages to the
/// connected client.
async fn process_stream(
    stream: TcpStream,
    peer_addr: SocketAddr,
    _server: &ServerHandle,
    net_rx: &mut mpsc::Receiver<ServerMessage>,
) {
    let (reader, mut writer) = stream.into_split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        tracing::info!(?peer_addr, "connection close by remote");
                        break;
                    },
                    Ok(_) => {
                        // TODO: Parse message and then relay it to the server via the handle.
                        print!("Received: {line}");
                        line.clear();
                    },
                    Err(error) => {
                        tracing::error!(?peer_addr, ?error, "failed to read message");
                        break;
                    },
                }
            }
            msg = net_rx.recv() => {
                match msg {
                    Some(_) => {
                        // TODO: Serialize the message and send to the client.
                        if let Err(error) = writer.write_all("TODO".as_bytes()).await {
                            tracing::error!(?peer_addr, ?error, "failed to send message");
                            break;
                        }
                    },
                    None => {
                        tracing::info!("server channel closed. exiting");
                        break;
                    }
                }
            }

        }
    }
}

/// Runs the server.
///
/// This will start the listen task to handle connections from the control node. When a control
/// node connects, it will start handling the communcation.
///
/// After the listen task is started, it will begin processing local requests.
///
/// This will run until the server's message channel is closed.
async fn run_server(mut server: Server, net_rx: mpsc::Receiver<ServerMessage>) {
    tracing::info!("server started");


    // Start listening for connections from the control node.
    // TODO: Port and maybe the address should be configurable.
    let handle = server.create_handle();
    task::spawn(async move {
        listen("127.0.0.1:7878", handle, net_rx).await;
    });

    while let Some(msg) = server.recv_msg().await {
        if let Err(error) = server.proc_msg(msg) {
            tracing::error!(?error, "failed to process server request");
        }
    }

    tracing::info!("server exited");
}

