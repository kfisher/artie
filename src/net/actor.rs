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
    /// `serial_number`:  The serial number of the drive the status update is for.
    ///
    /// `info`:  The optical drive information reported by the OS. If `None`, then the OS has
    /// stopped reporting information for the drive meaning the drive was disconnected or suffered
    /// some sort of hardware failure. 
    ///
    /// # Errors
    ///
    /// See [`drive::update_from_os`] for list of potential errors.
    async fn process_drive_status_update(
        &self,
        serial_number: &str,
        info: Option<OsOpticalDrive>,
    ) -> Result<()> {
        drive::update_from_os(&self.bus, serial_number, info).await
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
                let serial_number = drive.as_ref().unwrap().serial_number.clone(); // FIXME
                self.process_drive_status_update(&serial_number, drive).await
            }
        }
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
    // TODO
}
