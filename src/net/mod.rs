// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles network communication between the control and worker nodes.
//!
//! In multi-node configurations, there will be one control node and one or more worker nodes. The
//! control node will be the node the user interacts with and will be responsible for managing the
//! application data (e.g. database).
//!
//! Worker nodes will wait for commands from the control node. The primary purpose of worker nodes
//! will be to perform copy and transcode operations.
//!
//! Worker nodes will operate as a server and the control node will operate as a client connecting
//! to the workers.

mod actor;
pub mod client;
pub mod protocol;
pub mod server;

use serde::{Deserialize, Serialize};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::{Error, Result};
use crate::actor::Response;
use crate::bus;
use crate::drive::OsOpticalDrive;

/// Handle used to communicate with the client or server actor.
pub type Handle = crate::actor::Handle<Message>;

/// Represents a message containing a request from another application node or a request to be sent
/// to another application node.
#[derive(Debug)]
pub enum Message {
    Incoming(IncomingMessage),
    Outgoing(OutgoingMessage),
}

impl Message {
    /// Create a message being sent from a worker node to the control node.
    ///
    /// # Args
    ///
    /// `msg`:  The message to send to the control node.
    fn server(msg: protocol::Message) -> (Self, oneshot::Receiver<Result<()>>) {
        let (tx, rx) = oneshot::channel();
        let msg = OutgoingMessage {
            msg,
            worker: None,
            response: tx,
        };
        (Message::Outgoing(msg), rx)
    }
}

/// Message containing a request from another node.
#[derive(Debug)]
pub struct IncomingMessage {
    /// The message to received from the network.
    msg: protocol::Message,
}


/// Message for sending requests another node.
#[derive(Debug)]
pub struct OutgoingMessage {
    /// The message to send over the network.
    msg: protocol::Message,

    /// If applicable, the address and port of the worker node the message should be sent to.
    ///
    /// `Some` if message is meant for a worker node or `None` if meant for the control node.
    worker: Option<String>,

    /// Transmission end of the channel to send the response for the request.
    ///
    /// The response will be `Ok` if the message was sent successfully. If the message cannot be
    /// sent, the error will either be [`crate::Error::Disconnected`] or
    /// [`crate::Error::NetworkSend`].
    response: Response<()>,
}

impl OutgoingMessage {
    /// Serializes the message as JSON.
    ///
    /// This will automatically add a newline character to the end of the byte array before
    /// returning.
    ///
    /// # Errors
    ///
    /// [`crate::Error::SerdeJson`] if the bytes cannot be serialized.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        self.msg.serialize()
    }
}

/// Networking application settings.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    /// List of IP addresses for worker nodes.
    ///
    /// Only valid for the control node application instance.
    pub workers: Vec<String>,
}

/// Send an updated status of an optical drive to the control node.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The updated drive information. If `None`, then it is assumed that the drive is
/// disconnected.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_drive_status_update(
    bus: &bus::Handle,
    drive: Option<OsOpticalDrive>
) -> Result<()> {
    let msg = protocol::Message::DriveStatusUpdate { drive };
    let (msg, rx) = Message::server(msg);
    bus.send(msg).await?;
    rx.await?
}

/// Process communication from a network connection.
///
/// This will run until the connection is dropped or the actor closes the channel it uses to send
/// messages.
///
/// # Args
///
/// `stream`:  The stream for the network connection.
///
/// `peer_addr`:  The address of the application instance on the other side of the connection.
///
/// `actor`:  Handle to the client or server actor. Used to forward requests from the network
/// to the actor for further processing.
///
/// `net_rx`:  Receiving end of the channel used by the client or server actor to send messages to
/// the connected control node.
async fn process_stream(
    stream: TcpStream,
    peer_addr: &str,
    actor: &Handle,
    net_rx: &mut mpsc::Receiver<OutgoingMessage>,
) {
    let (reader, mut writer) = stream.into_split();

    let mut reader = BufReader::new(reader);
    let mut bytes = Vec::new();

    loop {
        tokio::select! {
            result = reader.read_until('\n' as u8, &mut bytes) => {
                match result {
                    Ok(0) => {
                        tracing::info!(?peer_addr, "connection close by remote");
                        break;
                    },
                    Ok(_) => {
                        let msg = protocol::Message::parse(&bytes).unwrap();
                        if let Err(error) = actor.send(msg.to_incoming_message()).await {
                            tracing::error!(?peer_addr, ?error, "failed to process message");
                            break;
                        }
                        bytes.clear();
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
                        if send(msg, peer_addr, &mut writer).await.is_err() {
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
async fn send(
    msg: OutgoingMessage,
    peer_addr: &str,
    writer: &mut OwnedWriteHalf,
) -> Result<()> {
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
