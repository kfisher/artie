// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use tracing::instrument;

use crate::{Error, Result};
use crate::error::ChannelError;

use super::DriveState;

/// Specifies the messages and responses for the optical drive actor.
///
/// Each message will have a channel that will be used by the actor to send the response. This will 
/// be a `oneshot` channel for a single response or a `mpsc` channel for streaming data such as
/// operation process information.
#[derive(Debug)]
pub enum DriveMessage {
    /// Request the current state of the optical drive.
    GetState {
        response: oneshot::Sender<DriveState>,
    },
}

/// Handle used to communicate with a [`DriveActor`] instance.
#[derive(Clone, Debug)]
pub struct DriveActorHandle {
    /// Transmission end of the channel used to send requests to the actor.
    tx: mpsc::Sender<DriveMessage>,
}

impl DriveActorHandle {
    /// Create a new [`DriveActorHandle`] instance.
    fn new(tx: mpsc::Sender<DriveMessage>) -> Self {
        Self { tx }
    }

    /// Get the current state of the optical drive.
    pub async fn get_state(&self) -> Result<DriveState> {
        let (tx, rx) = oneshot::channel();

        let msg = DriveMessage::GetState { response: tx };

        self.tx.send(msg).await
            .map_err(send_error)?;

        rx.await.map_err(oneshot_recv_error)
    }

    // TODO: Copy Request
    // - Message could have a full channel for sending back progress information
    // - Message would also require the copy perameters passed in via function.
    // - Need to figure out how to integrate with iced.
}

/// Actor used to perform copy operations and monitor the state of an optical 
#[derive(Debug)]
struct DriveActor {
    /// The serial number of the optical drive this actor is associated with.
    serial_number: String,

    /// Receiving end of the channel used to send requests to the actor.
    rx: mpsc::Receiver<DriveMessage>,
}

impl DriveActor {
    /// Create a [`DriveActor`] instance.
    fn new(serial_number: String, rx: mpsc::Receiver<DriveMessage>) -> Self {
        Self { serial_number, rx }
    }

    /// Create a [`DriveActor`] instance, start it, and return its handle.
    fn create(serial_number: String) -> DriveActorHandle {
        let (tx, rx) = mpsc::channel(10);

        let actor = DriveActor::new(serial_number, rx);
        tokio::spawn(run(actor));

        DriveActorHandle::new(tx)
    }

    /// Process a message that was received from the actor's communication channel.
    fn handle_message(&mut self, msg: DriveMessage) -> Result<()> {
        match msg {
            DriveMessage::GetState { response } => self.get_state(response),
        }
    }

    /// Process the request for getting the current state of the actor.
    fn get_state(&self, tx: oneshot::Sender<DriveState>) -> Result<()> {
        // TODO: For now, we assume always in an idle state. Once copying is implemented, will 
        //       need to account for that. When copying we done't want to make an OS call.
        let state = match super::get_optical_drive(&self.serial_number)? {
            Some(drive) => DriveState::Idle { disc: drive.disc },
            None => DriveState::Disconnected,
        };

        tx.send(state).map_err(|_| Error::DriveChannel { error: ChannelError::OneShotSend })?;

        Ok(())
    }
}

/// Create an application error from the provided tokio error.
fn oneshot_recv_error(e: oneshot::error::RecvError) -> Error {
    Error::DriveChannel { error: ChannelError::OneShotRecv(e) }
}

/// Create an application error from the provided tokio error.
fn send_error(e: mpsc::error::SendError<DriveMessage>) -> Error {
    Error::DriveChannel { error: ChannelError::Send(e) }
}

/// Runs the processing loop for the provided actor.
#[instrument]
async fn run(mut actor: DriveActor) {
    while let Some(msg) = actor.rx.recv().await {
        tracing::trace!(message=?msg, sn=actor.serial_number, "received message");
        if let Err(error) = actor.handle_message(msg) {
            tracing::trace!(sn=actor.serial_number, ?error, "Failed to process message.");
        }
    }
}

// TODO: TESTING
