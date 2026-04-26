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
//! Worker nodes will operate as a server (see [`server`]) and the control node will operate as a
//! client connecting to the workers (see [`client`]).

pub mod client;
pub mod protocol;
pub mod server;

use serde::{Deserialize, Serialize};

use tokio::sync::oneshot;

use crate::Result;
use crate::actor::{self, Response};
use crate::bus;
use crate::drive::OsOpticalDrive;

/// Handle used to communicate with the client or server actor.
pub type Handle = actor::Handle<Message>;

/// Message for sending requests to the client or server actor.
#[derive(Debug)]
pub struct Message {
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

impl Message {
    /// Create a message being sent from a worker node to the control node.
    ///
    /// # Args
    ///
    /// `msg`:  The message to send to the control node.
    fn server(msg: protocol::Message) -> (Self, oneshot::Receiver<Result<()>>) {
        let (tx, rx) = oneshot::channel();
        let msg = Message {
            msg,
            worker: None,
            response: tx,
        };
        (msg, rx)
    }

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

