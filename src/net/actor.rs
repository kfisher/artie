// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles network communication.
//!
//! This actor will run for both the server (worker side) and client (control side). It will handle
//! messages received from the network and relay them to the appropriate actor for processing. It
//! will also handle sending messages over the network when requested by an application actor.
//!
//! The actor can be initialized by calling [`init`] which will start the task for processing
//! requests for the actor. This is called when the server or client is initialized

use tokio::sync::mpsc;

use makemkv::{CopyCommandOutput, InfoCommandOutput};

use crate::Result;
use crate::actor;
use crate::bus;
use crate::drive::{self, OsOpticalDrive};
use crate::models::MediaLocation;
use crate::net::{Handle, IncomingMessage, Message, OutgoingMessage};
use crate::net::protocol;

/// Maximum number of queued messages.
const CHANNEL_BUFFER_SIZE: usize = 10;

/// Create a net actor instance.
///
/// This can be used for both the server actor or a client actor. This will start the task for
/// processing requests for the actor and then return a handle for sending requests to the actor as
/// well as the receiving end of the channel used by the actor to send messages over the network
/// to the connected peer.
pub fn init(name: &str, bus: &bus::Handle) -> (Handle, mpsc::Receiver<OutgoingMessage>)  {
    let (net_tx, net_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
    let msg_processor = MessageProcessor::new(bus.clone(), net_tx);
    (actor::create_and_run(name, msg_processor), net_rx)
}

/// Processes messages sent to the network actor.
struct MessageProcessor {
    /// Handle used to send messages to other actors via the message bus.
    bus: bus::Handle,

    /// Transmission end of the channel used to send messages over the network.
    net_tx: mpsc::Sender<OutgoingMessage>,
}

impl MessageProcessor {
    /// Create a new instance of the server message processor.
    ///
    /// # Args
    ///
    /// `bus`:  Handle used to send messages to other actors via the message bus.
    ///
    /// `net_tx`:  Transmission end of the channel used to send messages over the network.
    fn new(bus: bus::Handle, net_tx: mpsc::Sender<OutgoingMessage>) -> Self {
        Self { bus, net_tx }
    }

    /// Processes an incoming drive status update.
    ///
    /// # Args
    ///
    /// `drive`:  The optical drive information reported by the OS.
    ///
    /// `sender`:  The IP of the address peer that sent the update.
    ///
    /// # Errors
    ///
    /// See [`drive::update_from_os`] for list of potential errors.
    async fn process_drive_status_update(
        &self,
        drive: OsOpticalDrive,
        sender: String
    ) -> Result<()> {
        drive::update_from_os(&self.bus, drive, Some(sender)).await
    }

    /// Process a message that was received from the network.
    ///
    /// # Args
    ///
    /// `msg`:  The received messsage. The request within the message will be sent to the
    /// appropriate actor for processing.
    ///
    /// # Errors
    ///
    /// The potential errors will depend on the received message.
    async fn process_incoming(&self, incoming: IncomingMessage) -> Result<()> {
        match incoming.msg {
            protocol::Message::DriveStatusUpdate { drive } => {
                self.process_drive_status_update(drive, incoming.sender).await
            },
            protocol::Message::MakeMkvCancel { drive } => {
                self.process_makemkv_cancel(drive).await
            },
            protocol::Message::MakeMkvCopyComplete { drive, output } => {
                self.process_makemkv_copy_complete(drive, output).await
            },
            protocol::Message::MakeMkvFailed { drive, error } => {
                self.process_makemkv_failed(drive, error).await
            },
            protocol::Message::MakeMkvInfoComplete { drive, output } => {
                self.process_makemkv_info_complete(drive, output).await
            },
            protocol::Message::MakeMkvProgress { drive, op, op_prog, subop, subop_prog } => {
                self.process_makemkv_progress(drive, op, op_prog, subop, subop_prog).await
            },
            protocol::Message::RunMakeMkvCopy { drive, output_dir, log_file } => {
                self.process_makemkv_copy(drive, output_dir, log_file).await
            },
            protocol::Message::RunMakeMkvInfo { drive, log_file } => {
                self.process_makemkv_info(drive, log_file).await
            },
        }
    }

    /// Processes an incoming result of running the MakeMKV copy command.
    ///
    /// # Args
    ///
    /// `drive`:  The serial number of the drive that ran the MakeMKV command.
    ///
    /// `output`:  The output from the command.
    ///
    /// # Errors
    ///
    /// See [`drive::makemkv_copy_complete`] for list of potential errors.
    async fn process_makemkv_copy_complete(
        &self,
        drive: String,
        output: CopyCommandOutput,
    ) -> Result<()> {
        drive::makemkv_copy_complete(&self.bus, &drive, output).await
    }

    /// Process an incoming failure result of runnning a MakeMKV command.
    ///
    /// # Args
    ///
    /// `drive`:  The serial number of the drive that ran the MakeMKV command.
    ///
    /// `error`:  The commands error message.
    ///
    /// # Errors
    ///
    /// See [`drive::makemkv_failed`] for list of potential errors.
    async fn process_makemkv_failed(&self, drive: String, error: String) -> Result<()> {
        drive::makemkv_failed(&self.bus, &drive, error).await
    }

    /// Processes an incoming result of running the MakeMKV info command.
    ///
    /// # Args
    ///
    /// `drive`:  The serial number of the drive that ran the MakeMKV command.
    ///
    /// `output`:  The output from the command.
    ///
    /// # Errors
    ///
    /// See [`drive::makemkv_info_complete`] for list of potential errors.
    async fn process_makemkv_info_complete(
        &self,
        drive: String,
        output: InfoCommandOutput,
    ) -> Result<()> {
        drive::makemkv_info_complete(&self.bus, &drive, output).await
    }

    /// Processes an incoming request to cancel a running MakeMKV command.
    ///
    /// # Args
    ///
    /// `drive`:  The serial number of the drive running MakeMKV command.
    ///
    /// # Errors
    ///
    /// See [`drive::worker_makemkv_cancel`] for list of potential errors.
    async fn process_makemkv_cancel(&self, drive: String) -> Result<()> {
        drive::worker_makemkv_cancel(&self.bus, drive).await
    }

    /// Processes an incoming request to run the MakeMKV copy command.
    ///
    /// # Args
    ///
    /// `drive`:  The serial number of the drive to run the command on.
    ///
    /// # Errors
    ///
    /// See [`drive::worker_makemkv_copy`] for list of potential errors.
    async fn process_makemkv_copy(
        &self,
        drive: String,
        output_dir: MediaLocation,
        log_file: MediaLocation
    ) -> Result<()> {
        drive::worker_makemkv_copy(&self.bus, drive, output_dir, log_file).await
    }

    /// Processes an incoming progress update for a running MakeMKV command.
    ///
    /// # Args
    ///
    /// `drive`:  The serial number of the drive the command is running on.
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
    /// See [`drive::makemkv_progress`] for list of potential errors.
    async fn process_makemkv_progress(
        &self,
        drive: String,
        op: String,
        op_prog: u8,
        subop: String,
        subop_prog: u8
    ) -> Result<()> {
        drive::makemkv_progress(&self.bus, &drive, op, op_prog, subop, subop_prog).await
    }

    /// Processes an incoming request to run the MakeMKV info command.
    ///
    /// # Args
    ///
    /// `drive`:  The serial number of the drive to run the command on.
    ///
    /// # Errors
    ///
    /// See [`drive::worker_makemkv_info`] for list of potential errors.
    async fn process_makemkv_info(&self, drive: String, log_file: MediaLocation) -> Result<()> {
        drive::worker_makemkv_info(&self.bus, drive, log_file).await
    }

    /// Process a request to send a message over the network.
    ///
    /// `msg`:  The message to send over the network.
    ///
    /// # Errors
    ///
    /// [`crate::Error::NetworkChannelSend`] if the message could not be sent to the task
    /// responsible for handling network communication.
    async fn process_outgoing(&mut self, outgoing: OutgoingMessage) -> Result<()> {
        self.net_tx.send(outgoing).await
            .map_err(|e| e.into())
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Incoming(incoming) => self.process_incoming(incoming).await,
            Message::Outgoing(outgoing) => self.process_outgoing(outgoing).await,
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
