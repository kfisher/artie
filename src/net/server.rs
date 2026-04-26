// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Runs a server for a worker node.
//!
//! TODO

use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;

use crate::{Error, Result};
use crate::actor;
use crate::net::{Handle, Message};
use crate::task;

/// Maximum number of messages for a server actor that can be queued.
const SERVER_CHANNEL_BUFFER_SIZE: usize = 10;

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
    net_tx: mpsc::Sender<Message>,
}

impl MessageProcessor {
    /// Create a new instance of the server message processor.
    ///
    /// # Args
    ///
    /// `net_tx`:  Transmission end of the channel used to send messages to the connected control
    /// node.
    fn new(net_tx: mpsc::Sender<Message>) -> Self {
        Self { net_tx }
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        if msg.worker.is_some() {
            tracing::warn!("unexpected value for target worker in message");
        }
        self.net_tx.send(msg).await
            .map_err(|e| e.into())
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
async fn listen(addr: &str, server: Handle, mut net_rx: mpsc::Receiver<Message>) {
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(?error, ?addr, "failed to listen for connections");
            return;
        }
    };

    tracing::info!(?addr, "waiting for connection");
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, peer_addr)) => {
                        // We only support a single connected client at a time since there should
                        // only ever be one control node.
                        tracing::info!(?peer_addr, "client connected");
                        process_stream(stream, peer_addr, &server, &mut net_rx).await;
                        tracing::info!("client disconnected");
                    },
                    Err(error) => {
                        tracing::error!(?error, ?addr, "failed to accept connection");
                    }
                }
            }
            result = net_rx.recv() => {
                match result {
                    Some(msg) => {
                        tracing::trace!("attempted to send message when disconnected");
                        let _ = msg.response.send(Err(Error::Disconnected))
                            .inspect_err(|_| tracing::error!("failed to send response"));
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
    net_rx: &mut mpsc::Receiver<Message>,
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
            result = net_rx.recv() => {
                match result {
                    Some(msg) => {
                        if send(msg, &peer_addr, &mut writer).await.is_err() {
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

/// Helper function to send a message over the network.
///
/// # Args
///
/// `msg`:  The message to send over the network. This function will also send the success/fail
/// response back to the sender.
///
/// `peer_addr`:  The address the message is being sent to.
///
/// `writer`:  The writer used to send messages thru the TCP socket.
///
/// # Errors
///
/// [`Error::SerdeJson`] if the message cannot serialized.
///
/// [`Error::StdIo`] if an error occurs while trying to write to the network.
async fn send(msg: Message, peer_addr: &SocketAddr, writer: &mut OwnedWriteHalf) -> Result<()> {
    let result = {
        let bytes = match msg.serialize() {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::error!(?error, "failed to serialize message");
                return Err(error);
            }
        };

        writer.write_all(&bytes).await
            .map_err(|error| {
                tracing::error!(?peer_addr, ?error, "failed to send message");
                error.into()
            })
    };

    let reply = if result.is_ok() {
        Ok(())
    } else {
        Err(Error::NetworkSend)
    };

    // Ignore error sending the response so that the network connection isn't closed if sending the
    // message was successful, but sending the response was not. This could happen if the requester
    // doesn't wait for the response and the receiving end of the channel goes out of scope and is
    // dropped.
    let _ = msg.response.send(reply)
        .inspect_err(|_| tracing::error!("failed to send response"));

    result
}

