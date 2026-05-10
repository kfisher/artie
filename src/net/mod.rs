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
//!
//! # Control to Worker Requests
//!
//! The following helper methods can be used to send messages from the control node to a worker
//! node.
//!
//! - [`send_cancel_makemkv_op`]
//! - [`send_run_makemkv_copy`]
//! - [`send_run_makemkv_info`]
//!
//! # Worker to Control Requests
//!
//! The following helper methods can be used to send messages from a worker node to the control
//! node.
//!
//! - [`send_drive_status_update`]
//! - [`send_makemkv_copy_complete`]
//! - [`send_makemkv_failed`]
//! - [`send_makemkv_info_complete`]
//! - [`send_makemkv_progress`]

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

use makemkv::{CopyCommandOutput, InfoCommandOutput};

use crate::{Error, Result};
use crate::actor::Response;
use crate::bus;
use crate::drive::OsOpticalDrive;
use crate::models::MediaLocation;

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

    /// Create a messag being sent from the control node to a worker node.
    ///
    /// # Args
    ///
    /// `worker`:  The address of the worker node to send the message to.
    ///
    /// `msg`:  The message to send to the control node.
    fn worker(worker: &str, msg: protocol::Message) -> (Self, oneshot::Receiver<Result<()>>) {
        let (tx, rx) = oneshot::channel();
        let msg = OutgoingMessage {
            msg,
            worker: Some(worker.to_owned()),
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

    /// The IP address of the application instance that sent the request.
    sender: String,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    /// Address to listen on.
    ///
    /// Only valid for worker node instances of the application.
    #[serde(default = "Settings::default_addr")]
    pub listen_addr: String,

    /// Port to listen on.
    ///
    /// Only valid for worker node instances of the application.
    #[serde(default = "Settings::default_port")]
    pub listen_port: u16,

    /// List of IP addresses for worker nodes.
    ///
    /// Only valid for the control node application instance.
    #[serde(default)]
    pub workers: Vec<String>,
}

impl Settings {
    /// The address to listen on if one is not specified in the config.
    fn default_addr() -> String {
        String::from("127.0.0.1")
    }

    /// The port to listen on if one is not specified in the config.
    fn default_port() -> u16 {
        7878
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            listen_addr: Self::default_addr(),
            listen_port: Self::default_port(),
            workers: Default::default()
        }
    }
}

/// Send a request to a worker node to cancel a running MakeMKV operation.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `worker`:  The worker node to send the request to.
///
/// `drive`:  The serial number of the drive to running the MakeMKV command to cancel.
pub async fn send_cancel_makemkv_op(bus: &bus::Handle, worker: &str, drive: &str) -> Result<()> {
    let msg = protocol::Message::MakeMkvCancel { drive: drive.to_owned() };
    let (msg, rx) = Message::worker(worker, msg);
    bus.send(msg).await?;
    rx.await?
}

/// Send an updated status of an optical drive to the control node.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The updated drive information.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_drive_status_update(bus: &bus::Handle, drive: OsOpticalDrive) -> Result<()> {
    let msg = protocol::Message::DriveStatusUpdate { drive };
    let (msg, rx) = Message::server(msg);
    bus.send(msg).await?;
    rx.await?
}

/// Send a request to a worker node to run the MakMKV copy command.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The serial number of the drive to run the copy command on.
///
/// `output_dir`:  The media location of the directory the copied titles should be written to.
///
/// `log_file`:  The media location of the directory the output log of the MakeMKV command should
/// be written to.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_run_makemkv_copy(
    bus: &bus::Handle,
    worker: &str,
    drive: &str,
    output_dir: MediaLocation,
    log_file: MediaLocation,
) -> Result<()> {
    let msg = protocol::Message::RunMakeMkvCopy {
        drive: drive.to_owned(),
        output_dir,
        log_file,
    };
    let (msg, rx) = Message::worker(worker, msg);
    bus.send(msg).await?;
    rx.await?
}

/// Send a request to a worker node to run the MakMKV info command.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The serial number of the drive to run the info command on.
///
/// `log_file`:  The media location of the directory the output log of the MakeMKV command should
/// be written to.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_run_makemkv_info(
    bus: &bus::Handle,
    worker: &str,
    drive: &str,
    log_file: MediaLocation,
) -> Result<()> {
    let msg = protocol::Message::RunMakeMkvInfo {
        drive: drive.to_owned(),
        log_file,
    };
    let (msg, rx) = Message::worker(worker, msg);
    bus.send(msg).await?;
    rx.await?
}

/// Send the result of the MakeMKV copy command to the control node.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The serial number of the drive the MakeMKV command was run on.
///
/// `output`:  The output from the MakeMKV copy command.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_makemkv_copy_complete(
    bus: &bus::Handle,
    drive: &str,
    output: CopyCommandOutput,
) -> Result<()> {
    let msg = protocol::Message::MakeMkvCopyComplete {
        drive: drive.to_owned(),
        output,
    };
    let (msg, rx) = Message::server(msg);
    bus.send(msg).await?;
    rx.await?
}

/// Send the error information for a failed MakeMKV command to the control node.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The serial number of the drive the MakeMKV command was run on.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_makemkv_failed(
    bus: &bus::Handle,
    drive: &str,
    error: String,
) -> Result<()> {
    let msg = protocol::Message::MakeMkvFailed {
        drive: drive.to_owned(),
        error,
    };
    let (msg, rx) = Message::server(msg);
    bus.send(msg).await?;
    rx.await?
}

/// Send the result of the MakeMKV info command to the control node.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The serial number of the drive the MakeMKV command was run on.
///
/// `output`:  The output from the MakeMKV info command.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_makemkv_info_complete(
    bus: &bus::Handle,
    drive: &str,
    output: InfoCommandOutput,
) -> Result<()> {
    let msg = protocol::Message::MakeMkvInfoComplete {
        drive: drive.to_owned(),
        output,
    };
    let (msg, rx) = Message::server(msg);
    bus.send(msg).await?;
    rx.await?
}

/// Send the current progress output from a MakeMKV command to the control node.
///
/// # Args
///
/// `bus`:  Handle for sending the request to the network actor.
///
/// `drive`:  The serial number of the drive the MakeMKV command was run on.
///
/// `op`:  Title of the current operation.
///
/// `op_prog`:  Progress of the current operation.
///
/// `subop`:  Title of the current suboperation.
///
/// `subop_prog`:  Progress of the current suboperation.
///
/// # Errors
///
/// [`crate::Error::Disconnected`] if the node is not connected to the control node.
///
/// [`crate::Error::NetworkSend`] if the message could not be sent.
pub async fn send_makemkv_progress(
    bus: &bus::Handle,
    drive: &str,
    op: String,
    op_prog: u8,
    subop: String,
    subop_prog: u8,
) -> Result<()> {
    let msg = protocol::Message::MakeMkvProgress {
        drive: drive.to_owned(),
        op,
        op_prog,
        subop,
        subop_prog,
    };
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
            result = reader.read_until(b'\n', &mut bytes) => {
                match result {
                    Ok(0) => {
                        tracing::info!(?peer_addr, "connection close by remote");
                        break;
                    },
                    Ok(_) => {
                        let msg = protocol::Message::parse(&bytes).unwrap();
                        if let Err(error) = actor.send(msg.incoming_message(peer_addr)).await {
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
                        tracing::info!("receiving channel closed. exiting");
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

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
