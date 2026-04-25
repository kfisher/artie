// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Runs a server for a worker node.
//!
//! TODO

use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::actor;
use crate::net::{Handle, Message};
use crate::task;

/// Maxium number of messages for a server actor that can be queued.
const SERVER_CHANNEL_BUFFER_SIZE: usize = 10;

// TODO
enum TmpMsg {
}

/// Create the the server actor.
///
/// This will create the actor, spawn the task for processing requests from the application and
/// spawn the task for listening for and processing messages from the control node,
pub fn init() -> Handle {
    // Channel used for the server actor to send messages over the network.
    let (net_tx, net_rx) = mpsc::channel(SERVER_CHANNEL_BUFFER_SIZE);

    let msg_processor = MessageProcessor::new(net_tx);

    let handle = actor::create_and_run("server", msg_processor);

    // TODO: Listen address and port should be configurable.
    let handle_clone = handle.clone();
    task::spawn(async move {
        listen("127.0.0.1:7878", handle_clone, net_rx).await;
    });

    handle
}

/// Processes messages sent to the server actor.
struct MessageProcessor {
    /// Transmission end of the channel used to send messages to the connected control node.
    net_tx: mpsc::Sender<TmpMsg>,
}

impl MessageProcessor {
    /// Create a new instance of the server message processor.
    ///
    /// # Args
    ///
    /// `net_tx`:  Transmission end of the channel used to send messages to the connected control
    /// node.
    fn new(net_tx: mpsc::Sender<TmpMsg>) -> Self {
        Self { net_tx }
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, _msg: Message) -> crate::Result<()> {
        todo!()
    }
}

/// Listen for connections from the control node.
///
/// # Args
///
/// `addr`:  The address and port to listen on.
///
/// `bus`:  Handle used to send messages to the server actor. 
///
/// `net_rx`:  Receiving end of the channel used by the server message processor to send messages
/// to the connected control node.
async fn listen(addr: &str, server: Handle, mut net_rx: mpsc::Receiver<TmpMsg>) {
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

/// Process communication with a connected control node.
///
/// This will run until the connect control node drops the connection or the server actor closes
/// the channel it uses to send messages.
///
/// # Args
///
/// `stream`:  The stream for the connected control node.
///
/// `peer_addr`:  The address of the connected control node.
///
/// `server`:  Handle to the server actor. Used to forward messages from the connected control node
/// to the actor for processing.
///
/// `net_rx`:  Receiving end of the channel used by the server actor to send messages to the
/// connected control node.
async fn process_stream(
    stream: TcpStream,
    peer_addr: SocketAddr,
    _server: &Handle,
    net_rx: &mut mpsc::Receiver<TmpMsg>,
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

