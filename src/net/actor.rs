// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use tokio::sync::mpsc;

use crate::Result;
use crate::actor;
use crate::bus;
use crate::drive::{self, OsOpticalDrive};
use crate::net::{Handle, IncomingMessage, Message, OutgoingMessage};
use crate::net::protocol;

/// Maximum number of queued messages.
const CHANNEL_BUFFER_SIZE: usize = 10;

// TODO
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
    /// `info`:  The optical drive information reported by the OS.
    ///
    /// # Errors
    ///
    /// See [`drive::update_from_os`] for list of potential errors.
    async fn process_drive_status_update(&self, drive: OsOpticalDrive,) -> Result<()> {
        drive::update_from_os(&self.bus, drive).await
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
            protocol::Message::CancelMakeMkvOperation => {
                todo!()
            },
            protocol::Message::DriveStatusUpdate { drive } => {
                self.process_drive_status_update(drive).await
            },
            protocol::Message::RunMakeMkvCopy { device_path: _, output_dir: _, log_file: _ } => {
                todo!()
            },
            protocol::Message::RunMakeMkvInfo { device_path: _, log_file: _ } => {
                todo!()
            },
            protocol::Message::MakeMkvCopyResult { log: _ } => {
                todo!()
            },
            protocol::Message::MakeMkvInfoResult { disc_info: _, log: _ } => {
                todo!()
            },
            protocol::Message::MakeMkvProgress { op: _, op_prog: _, subop: _, subop_prog: _ } => {
                todo!()
            },
        }
    }

            // protocol::Message::DriveStatusUpdate { drive } => {
            // }

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
    // TODO
}
